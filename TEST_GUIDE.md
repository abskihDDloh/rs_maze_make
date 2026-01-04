# move_next モジュールテスト実行ガイド

## テストの概要

`tests/move_next_test.rs` は、`move_next` モジュールの2つの主要な関数をテストします：

- `get_adjacent_unused_extendable_pillar()` - 隣接する未使用の拡張可能な柱（開始点）の取得
- `path_to_wall()` - 2つの柱間の経路セルをWALL（壁）に変更

## テスト手順

テストは以下の11ステップで構成されています：

1. **DB初期化** - 5x5グリッドでデータベースを初期化
2. **セル状態確認1** - MAZE_CELL_STATUS_VIEWの内容をログ出力
3. **開始点A選択** - `select_random_start_point_from_db()`で開始点を取得
4. **セル状態確認2** - MAZE_CELL_STATUS_VIEWの内容をログ出力
5. **開始点状態確認1** - USED_START_POINTS_VIEWの内容をログ出力
6. **開始点B取得** - `get_adjacent_unused_extendable_pillar()`で次の開始点を取得
7. **セル状態確認3** - MAZE_CELL_STATUS_VIEWの内容をログ出力
8. **開始点状態確認2** - USED_START_POINTS_VIEWの内容をログ出力
9. **PATHをWALLに変更** - `path_to_wall()`で中間点をWALLに変更
10. **変更確認** - 中間点がWALLになっていることをアサート確認
11. **最終確認** - 開始点A,BがDIRECT_CONNECTで存在することをアサート確認

## セットアップ手順

### 前提条件

- MySQLサーバーがインストールされ、起動していること
- Rustとcargoがインストールされていること
- プロジェクトルートディレクトリ（`/home/normal/rs_maze_maker`）に移動していること

### ステップ1: 環境変数の設定

```bash
# .env.example を参考にして .env ファイルを作成
cp .env.example .env
```

`.env` ファイルを編集して、環境に合わせて `DATABASE_URL` を設定：

```env
# MySQL接続URL（例）
DATABASE_URL=mysql://root:password@localhost:3306/maze_maker_db
```

### ステップ2: MySQLデータベースの準備

MySQLクライアントで以下を実行：

```sql
-- データベースの作成（存在しない場合）
CREATE DATABASE IF NOT EXISTS maze_maker_db CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

-- ユーザーの確認（必要に応じて作成）
-- 例: CREATE USER 'your_user'@'localhost' IDENTIFIED BY 'your_password';
-- GRANT ALL ON maze_maker_db.* TO 'your_user'@'localhost';
```

### ステップ3: データベーススキーマの初期化

テストが実行時にスキーマを初期化するため、別途の手動スキーマ作成は通常不要です。
ただし、既存のスキーマがある場合は、テスト前にクリーニングが必要な場合があります。

## テストの実行

### 簡単な実行方法

```bash
# テストを実行（ログ出力付き）
cargo test --test move_next_test -- --nocapture --ignored
```

### 詳細ログ付き実行

```bash
# RUSTログレベルをDebugに設定して実行
RUST_LOG=debug cargo test --test move_next_test -- --nocapture --ignored
```

### 特定のテスト関数のみ実行

```bash
cargo test test_get_adjacent_unused_extendable_pillar_and_path_to_wall -- --nocapture --ignored
```

## トラブルシューティング

### エラー: `ConnectionAcquire(Timeout)`

**原因**: データベース接続がタイムアウトしています。

**解決方法**:

1. **MySQLが起動しているか確認**
   ```bash
   # MySQLサーバーの状態確認（Linux/Mac）
   sudo systemctl status mysql
   
   # または MySQL CLIで確認
   mysql -u root -p -e "SELECT 1"
   ```

2. **DATABASE_URLが正しいか確認**
   ```bash
   # .env ファイルに設定された DATABASE_URL を確認
   cat .env | grep DATABASE_URL
   ```

3. **接続情報が正しいか確認**
   ```bash
   # 接続テスト
   mysql -u your_user -p -h localhost -e "SELECT 1"
   ```

4. **ファイアウォール/ネットワーク設定を確認**
   - MySQLがlocalhostのポート3306でリッスンしているか確認
   - セキュリティソフトがブロックしていないか確認

### エラー: `Unknown database 'maze_maker_db'`

**原因**: 指定されたデータベースが存在しません。

**解決方法**:

```bash
# MySQLで以下を実行
mysql -u root -p
```

```sql
CREATE DATABASE IF NOT EXISTS maze_maker_db CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
```

### エラー: `Access denied for user`

**原因**: データベースユーザーの認証に失敗しています。

**解決方法**:

1. `DATABASE_URL` の認証情報が正しいか確認
2. MySQLユーザーが存在するか確認
3. 必要に応じてユーザーを作成：
   ```sql
   CREATE USER 'your_user'@'localhost' IDENTIFIED BY 'your_password';
   GRANT ALL ON maze_maker_db.* TO 'your_user'@'localhost';
   FLUSH PRIVILEGES;
   ```

### テストがスキップされる

**原因**: 環境変数 `DATABASE_URL` が設定されていないか、接続に失敗しています。

**解決方法**:

テストは `#[ignore]` 属性により、`--ignored` フラグを付けないとスキップされます。
必ず以下のように実行してください：

```bash
cargo test --test move_next_test -- --nocapture --ignored
```

## テスト出力の例

成功時の出力例：

```
Step 1: Initializing database with 5x5 grid
Step 2: Logging MAZE_CELL_STATUS_VIEW after initialization
  Cell status: x=1, y=1, type=PILLAR, owner_thread_id=None
  Cell status: x=1, y=2, type=PATH, owner_thread_id=None
  ...
Step 3: Selecting random start point A
  Start point A: (1, 1)
...
✓ All test assertions passed!
```

## 詳細情報

- **テストファイル**: `tests/move_next_test.rs`
- **テスト対象**: `src/maze/move_next.rs`
- **関連モジュール**:
  - `src/database/connector.rs` - データベース接続
  - `src/database/initializer.rs` - データベーススキーマ初期化
  - `src/maze/independent_transaction_routines.rs` - トランザクション処理

## 参考リンク

- [SeaORM公式ドキュメント](https://www.sea-ql.org/SeaORM/)
- [Tokioドキュメント](https://tokio.rs/)
- [MySQLドキュメント](https://dev.mysql.com/doc/)
