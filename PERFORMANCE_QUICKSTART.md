# パフォーマンス最適化クイックスタート

## maze_thread_function()が遅い問題の解決方法

### 最も効果的な方法：データベースインデックスの追加

#### ステップ1: インデックスを追加
```bash
# データベースに接続（パスワードを入力）
mysql -u your_user -p MAZEMAKE < schema/03_performance_indexes.sql
```

#### ステップ2: 効果を確認
```bash
# 再ビルド
./build_all.sh

# 実行時間を計測
time ./target/release/make_maze -x 513 -y 513 -t 4
```

### 期待される効果
- **513x513グリッド**: 10-30分 → **2-5分** （3-10倍高速化）
- **1023x1023グリッド**: 数時間 → **10-30分**

### トラブルシューティング

#### インデックス追加でエラーが出る場合
```sql
-- 既存のインデックスを確認
SHOW INDEX FROM MAZE_FIELD;

-- 重複がある場合は先に削除
ALTER TABLE MAZE_FIELD DROP INDEX idx_cell_type_owner;
```

#### それでも遅い場合
1. `OPERATIONS_PER_COMMIT`を調整（[maze_thread.rs#L25](src/bin/make_maze/maze/maze_thread.rs)）
   - 値を大きく（50-100）すると、トランザクションオーバーヘッドが減る
   - 値を小さく（10-15）すると、ロック競合が減る

2. データベースの統計情報を更新
   ```sql
   ANALYZE TABLE MAZE_FIELD;
   ANALYZE TABLE MAZE_CELL;
   ```

3. クエリプランを確認
   ```sql
   EXPLAIN SELECT * FROM UNUSED_START_POINTS_VIEW LIMIT 10;
   ```

### さらなる最適化

詳細は [PERFORMANCE_OPTIMIZATION.md](PERFORMANCE_OPTIMIZATION.md) を参照してください。
