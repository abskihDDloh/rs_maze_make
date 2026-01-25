# maze_thread_function() パフォーマンス最適化ガイド

## 特定されたボトルネック

### 1. トランザクションのオーバーヘッド（最大の問題）
**現状**: 内側のループで毎回 `db.begin()` → `commit()`/`rollback()` を実行
- 大規模な迷路（513x513）では数十万回のトランザクションが発生
- 各トランザクションにはネットワーク往復・ロック取得・コミットログのオーバーヘッドがある

**最適化案A**: バッチトランザクション
```rust
const TRANSACTION_BATCH_SIZE: usize = 10; // 10回の操作ごとにコミット
let mut operation_count = 0;
let mut txn = db.begin().await?;

loop {
    // ... existing logic ...
    
    operation_count += 1;
    if operation_count >= TRANSACTION_BATCH_SIZE {
        txn.commit().await?;
        txn = db.begin().await?;
        operation_count = 0;
    }
}
```

### 2. check_extendable_pillar_existance() の頻繁な呼び出し
**現状**: 外側のループで毎回 COUNT クエリを実行
- 513x513 の場合、約130,000回の不要なCOUNTクエリ

**最適化案B**: キャッシュとエラーハンドリングによる遅延チェック
```rust
// 外側のループの最初でチェックを削除し、select_random_start_point_from_db() の
// エラーでループを抜ける
loop {
    let tid = MazeThreadIdentifier::new();
    
    // このエラーが発生したら未使用の柱がない
    let start_point = match select_random_start_point_from_db(db, &tid).await {
        Ok(p) => p,
        Err(_) => break, // 未使用の開始点がない
    };
    // ...
}
```

### 3. データベースクエリの最適化

**問題**: 
- `get_all_unused_pillars()` が毎回全レコードを取得
- `check_unused_pillars()` が複数の OR 条件を持つ重いクエリ

**最適化案C**: インデックスの確認と追加
```sql
-- UNUSED_START_POINTS_VIEW の基盤テーブルに複合インデックスを追加
CREATE INDEX idx_maze_field_type_owner ON MAZE_FIELD(CELL_TYPE, CELL_OWNER_THREAD_ID);
CREATE INDEX idx_maze_cell_xy ON MAZE_CELL(X, Y);
```

### 4. エラーハンドリングの非効率性
**現状**: エラー時にスタックをpopして1つ戻るだけ
- 連続失敗時に何度も同じパターンを繰り返す可能性

**最適化案D**: 連続失敗カウンタ（既に実装済み）
```rust
let mut consecutive_failures = 0;
const MAX_CONSECUTIVE_FAILURES: u32 = 3;
```

## 推奨実装順序

### フェーズ1: 即効性のある改善（工数: 小）
1. ✅ 連続失敗カウンタの実装（完了）
2. check_extendable_pillar_existance() の削除とエラーベースの制御
3. バッチトランザクション（10-20操作ごと）

### フェーズ2: データベース最適化（工数: 中）
4. インデックスの追加と確認
5. ビュークエリの最適化（UNUSED_START_POINTS_VIEW など）
6. 接続プールの設定確認

### フェーズ3: アーキテクチャ改善（工数: 大）
7. 並列処理の導入（複数スレッドで同時実行）
8. メモリ内キャッシュの利用
9. 一括更新APIの実装

## ベンチマーク目標

### 現状（推定）
- 513x513 グリッド: 10-30分
- トランザクション数: ~130,000回
- データベースクエリ数: ~500,000回

### 目標（フェーズ1完了後）
- 513x513 グリッド: 2-5分（5-10倍高速化）
- トランザクション数: ~13,000回（バッチサイズ10の場合）
- データベースクエリ数: ~200,000回

### 最終目標（フェーズ3完了後）
- 513x513 グリッド: <1分
- 並列度: 4-8スレッド
