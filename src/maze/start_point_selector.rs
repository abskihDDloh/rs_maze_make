use rand::Rng;
use sea_orm::{ConnectionTrait, DbConn, EntityTrait, Statement, TransactionTrait};

use crate::maze::{maze_point::MazePoint, maze_thread_identifier::MazeThreadIdentifier};

pub(in crate::maze) async fn select_random_start_point_from_db(
    db: &DbConn,
    tid: &MazeThreadIdentifier,
) -> Result<MazePoint, Box<dyn std::error::Error>> {
    let txn = db.begin().await?;
    // UNUSED_START_POINTS_VIEWを全件取得する。
    let unused_start_points: Vec<crate::database::entities::unused_start_points_view::Model> =
        crate::database::entities::unused_start_points_view::Entity::find()
            .all(&txn)
            .await?;
    if unused_start_points.is_empty() {
        return Err("No unused start points available.".into());
    }
    // ランダムに1件選択する。
    let mut rng = rand::rng();
    let random_index = rng.random_range(0..unused_start_points.len());
    let selected_point = &unused_start_points[random_index];
    let x = selected_point.x;
    let y = selected_point.y;
    let tid_str = tid.as_str();
    let unix_time = tid.unix_time_as_date_time_utc()?;
    // 選択したスタートポイントとMazeThreadIdentifierの内容をADD_NEW_THREADプロシージャを使ってTHERAD_LISTテーブルとMAZE_FIELDテーブルに登録する。
    let sql = "CALL ADD_NEW_THREAD(?, ?, ?, ?)";
    txn.execute(Statement::from_sql_and_values(
        sea_orm::DbBackend::MySql,
        sql,
        vec![
            x.into(),
            y.into(),
            tid_str.to_string().into(),
            unix_time.into(),
        ],
    ))
    .await?;
    txn.commit().await?;
    Ok(MazePoint::new(x, y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // DATABASE_URL が必要なため
    async fn test_select_random_start_point_five_times() {
        // DB接続を確立
        let db = crate::database::connector::establish_connection(None)
            .await
            .expect("Failed to connect to database");

        // DB初期化（5x5グリッド）
        crate::database::initializer::initialize_db(&db, 5, 5)
            .await
            .expect("Failed to initialize database");

        // select_random_start_point_from_db を5回呼び出し
        let mut selected_points = Vec::new();
        for i in 0..5 {
            let tid = MazeThreadIdentifier::new();
            match select_random_start_point_from_db(&db, &tid).await {
                Ok(point) => {
                    eprintln!(
                        "Call {}: Successfully selected point ({}, {})",
                        i + 1,
                        point.x(),
                        point.y()
                    );
                    selected_points.push(point);
                }
                Err(e) => {
                    panic!("Call {}: Failed to select start point: {}", i + 1, e);
                }
            }
        }

        // 検証：5個のポイントがすべて正常に取得できたこと
        assert_eq!(
            selected_points.len(),
            5,
            "Should have successfully selected 5 start points"
        );

        // すべての取得したポイントが有効な座標であること
        for point in &selected_points {
            assert!(point.x() < u64::MAX, "X coordinate should be valid");
            assert!(point.y() < u64::MAX, "Y coordinate should be valid");
        }

        eprintln!(
            "Successfully selected {} start points without errors",
            selected_points.len()
        );
    }
}
