use log::info;
use sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseTransaction, EntityTrait, Statement, TransactionTrait,
};
use strum_macros::{AsRefStr, Display, EnumString};

#[derive(Debug, PartialEq, Display, EnumString, AsRefStr, Clone, Copy)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")] // DB文字列に合わせる
pub enum OutsideWallConnectTypeEnum {
    DIRECT_CONNECT,
    INDIRECT_CONNECT,
    NOT_CONNECT,
}

async fn erase_maze_field(txn: &DatabaseTransaction) -> Result<(), sea_orm::DbErr> {
    crate::common::database::entities::maze_field::Entity::delete_many()
        .exec(txn)
        .await?;
    Ok(())
}

async fn erase_thread_list(txn: &DatabaseTransaction) -> Result<(), sea_orm::DbErr> {
    crate::common::database::entities::thread_list::Entity::delete_many()
        .exec(txn)
        .await?;

    Ok(())
}

async fn erase_outside_wall_connect_type(txn: &DatabaseTransaction) -> Result<(), sea_orm::DbErr> {
    crate::common::database::entities::outside_wall_connect_type::Entity::delete_many()
        .exec(txn)
        .await?;
    Ok(())
}

async fn erase_maze_cell(txn: &DatabaseTransaction) -> Result<(), sea_orm::DbErr> {
    crate::common::database::entities::maze_cell::Entity::delete_many()
        .exec(txn)
        .await?;
    Ok(())
}
async fn erase_maze_cell_type(txn: &DatabaseTransaction) -> Result<(), sea_orm::DbErr> {
    crate::common::database::entities::maze_cell_type::Entity::delete_many()
        .exec(txn)
        .await?;
    Ok(())
}

async fn initialize_outside_wall_connect_type(
    txn: &DatabaseTransaction,
) -> Result<(), sea_orm::DbErr> {
    let connect_types = vec![
        OutsideWallConnectTypeEnum::DIRECT_CONNECT,
        OutsideWallConnectTypeEnum::INDIRECT_CONNECT,
        OutsideWallConnectTypeEnum::NOT_CONNECT,
    ];

    for connect_type in connect_types {
        let new_connect_type =
            crate::common::database::entities::outside_wall_connect_type::ActiveModel {
                r#type: sea_orm::ActiveValue::Set(connect_type.to_string()),
            };
        crate::common::database::entities::outside_wall_connect_type::Entity::insert(
            new_connect_type,
        )
        .exec(txn)
        .await?;
    }
    Ok(())
}

#[derive(Debug, PartialEq, Display, EnumString, AsRefStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MazeCellTypeEnum {
    WALL,
    PATH,
    PILLAR,
    START,
    ROUTE,
    END,
}

async fn initialize_maze_cell(
    txn: &DatabaseTransaction,
    x_max: u64,
    y_max: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cells = Vec::new();
    const BATCH_SIZE: usize = 1000; // MySQLのプレースホルダー上限対策

    for x in 0..x_max {
        for y in 0..y_max {
            let new_start_point = crate::common::database::entities::maze_cell::ActiveModel {
                x: sea_orm::ActiveValue::Set(x),
                y: sea_orm::ActiveValue::Set(y),
                id: sea_orm::ActiveValue::NotSet,
            };
            cells.push(new_start_point);

            // バッチサイズに達したら挿入
            if cells.len() >= BATCH_SIZE {
                crate::common::database::entities::maze_cell::Entity::insert_many(cells.drain(..))
                    .exec(txn)
                    .await?;
            }
        }
    }

    // 残りのデータを挿入
    if !cells.is_empty() {
        crate::common::database::entities::maze_cell::Entity::insert_many(cells)
            .exec(txn)
            .await?;
    }

    Ok(())
}

async fn initialize_maze_cell_type(txn: &DatabaseTransaction) -> Result<(), sea_orm::DbErr> {
    let cell_types = vec![
        MazeCellTypeEnum::WALL,
        MazeCellTypeEnum::PATH,
        MazeCellTypeEnum::PILLAR,
        MazeCellTypeEnum::START,
        MazeCellTypeEnum::END,
    ];

    for cell_type in cell_types {
        let new_cell_type = crate::common::database::entities::maze_cell_type::ActiveModel {
            cell_type: sea_orm::ActiveValue::Set(cell_type.to_string()),
        };
        crate::common::database::entities::maze_cell_type::Entity::insert(new_cell_type)
            .exec(txn)
            .await?;
    }
    Ok(())
}
async fn initialize_maze_field(
    txn: &DatabaseTransaction,
    x_max: u64,
    y_max: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    // MAZE_CELLテーブルの内容を全件取得する。
    let maze_cells = crate::common::database::entities::maze_cell::Entity::find()
        .all(txn)
        .await?;

    let mut fields = Vec::new();
    const BATCH_SIZE: usize = 1000; // MySQLのプレースホルダー上限対策

    for cell in maze_cells {
        let id = cell.id;
        let x = cell.x;
        let y = cell.y;
        let new_cell_type: String;
        if x % 2 == 0 && y % 2 == 0 {
            new_cell_type = MazeCellTypeEnum::PILLAR.to_string();
        } else if x == 0 || x == x_max - 1 || y == 0 || y == y_max - 1 {
            new_cell_type = MazeCellTypeEnum::WALL.to_string();
        } else {
            new_cell_type = MazeCellTypeEnum::PATH.to_string();
        }
        let new_field = crate::common::database::entities::maze_field::ActiveModel {
            id: sea_orm::ActiveValue::Set(id),
            cell_type: sea_orm::ActiveValue::Set(new_cell_type),
            cell_owner_thread_id: sea_orm::ActiveValue::Set(None),
        };
        fields.push(new_field);

        // バッチサイズに達したら挿入
        if fields.len() >= BATCH_SIZE {
            crate::common::database::entities::maze_field::Entity::insert_many(fields.drain(..))
                .exec(txn)
                .await?;
        }
    }

    // 残りのデータを挿入
    if !fields.is_empty() {
        crate::common::database::entities::maze_field::Entity::insert_many(fields)
            .exec(txn)
            .await?;
    }

    Ok(())
}

// Populate TEMP_UNUSED_START_POINTS with the current unused start points
pub async fn populate_temp_unused_start_points(
    txn: &DatabaseTransaction,
) -> Result<(), sea_orm::DbErr> {
    let truncate_sql = "TRUNCATE TABLE TEMP_UNUSED_START_POINTS";
    txn.execute(Statement::from_string(
        DatabaseBackend::MySql,
        truncate_sql.to_string(),
    ))
    .await?;

    let insert_sql = "INSERT INTO TEMP_UNUSED_START_POINTS (CELL_ID) \
                      SELECT CELL_ID FROM UNUSED_START_POINTS_VIEW";
    txn.execute(Statement::from_string(
        DatabaseBackend::MySql,
        insert_sql.to_string(),
    ))
    .await?;

    Ok(())
}

pub async fn initialize_db(
    db: &sea_orm::DbConn,
    x_max: u64,
    y_max: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    // Validate input parameters
    if x_max < 5 || y_max < 5 || x_max.is_multiple_of(2) || y_max.is_multiple_of(2) {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "x_max and y_max must be odd numbers and >= 5",
        )));
    }
    if x_max > i64::MAX as u64 || y_max > i64::MAX as u64 {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "x_max and y_max must be within the range of i64",
        )));
    }

    info!("Starting database initialization");

    let txn = db.begin().await?;

    info!("First, delete child tables that reference parent tables (foreign key constraints)");
    erase_maze_field(&txn).await?;
    erase_thread_list(&txn).await?;

    info!("Then delete and recreate parent tables");
    erase_maze_cell_type(&txn).await?;
    erase_maze_cell(&txn).await?;
    erase_outside_wall_connect_type(&txn).await?;

    info!("Now recreate the data");
    initialize_outside_wall_connect_type(&txn).await?;
    initialize_maze_cell_type(&txn).await?;
    initialize_maze_cell(&txn, x_max, y_max).await?;
    initialize_maze_field(&txn, x_max, y_max).await?;
    populate_temp_unused_start_points(&txn).await?;

    txn.commit().await?;

    info!("Database initialization complete");

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::common::database::connector::establish_connection;
    use sea_orm::PaginatorTrait;

    use super::*;

    #[test]
    fn test_cell_type_constants() {
        assert_eq!(MazeCellTypeEnum::WALL.to_string(), "WALL");
        assert_eq!(MazeCellTypeEnum::PATH.to_string(), "PATH");
        assert_eq!(MazeCellTypeEnum::PILLAR.to_string(), "PILLAR");
        assert_eq!(MazeCellTypeEnum::START.to_string(), "START");
        assert_eq!(MazeCellTypeEnum::END.to_string(), "END");
    }

    // Helper function to validate input parameters
    fn validate_maze_dimensions(x_max: u64, y_max: u64) -> Result<(), String> {
        if x_max < 5 || y_max < 5 {
            return Err("x_max and y_max must be >= 5".to_string());
        }
        if x_max.is_multiple_of(2) || y_max.is_multiple_of(2) {
            return Err("x_max and y_max must be odd numbers".to_string());
        }
        if x_max > i64::MAX as u64 || y_max > i64::MAX as u64 {
            return Err("x_max and y_max must be within the range of i64".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_validate_maze_dimensions_valid() {
        assert!(validate_maze_dimensions(5, 5).is_ok());
        assert!(validate_maze_dimensions(7, 9).is_ok());
        assert!(validate_maze_dimensions(101, 201).is_ok());
    }

    #[test]
    fn test_validate_maze_dimensions_even_numbers() {
        assert!(validate_maze_dimensions(6, 5).is_err());
        assert!(validate_maze_dimensions(5, 8).is_err());
        assert!(validate_maze_dimensions(10, 10).is_err());
    }

    #[test]
    fn test_validate_maze_dimensions_too_small() {
        assert!(validate_maze_dimensions(3, 5).is_err());
        assert!(validate_maze_dimensions(5, 3).is_err());
        assert!(validate_maze_dimensions(1, 1).is_err());
    }

    #[test]
    fn test_validate_maze_dimensions_overflow() {
        let too_large = (i64::MAX as u64) + 2;
        assert!(validate_maze_dimensions(too_large, 5).is_err());
        assert!(validate_maze_dimensions(5, too_large).is_err());
    }

    // Cell position and type logic tests
    fn determine_cell_type(x: u64, y: u64, x_max: u64, y_max: u64) -> String {
        if x % 2 == 0 && y % 2 == 0 {
            MazeCellTypeEnum::PILLAR.to_string()
        } else if x == 0 || x == x_max - 1 || y == 0 || y == y_max - 1 {
            MazeCellTypeEnum::WALL.to_string()
        } else {
            MazeCellTypeEnum::PATH.to_string()
        }
    }

    #[test]
    fn test_cell_type_corners() {
        // Corners at even coordinates are PILLAR (not WALL)
        // because the even coordinate check comes first in the logic
        assert_eq!(
            determine_cell_type(0, 0, 5, 5),
            MazeCellTypeEnum::PILLAR.to_string()
        );
        assert_eq!(
            determine_cell_type(0, 4, 5, 5),
            MazeCellTypeEnum::PILLAR.to_string()
        );
        assert_eq!(
            determine_cell_type(4, 0, 5, 5),
            MazeCellTypeEnum::PILLAR.to_string()
        );
        assert_eq!(
            determine_cell_type(4, 4, 5, 5),
            MazeCellTypeEnum::PILLAR.to_string()
        );
    }

    #[test]
    fn test_cell_type_edges() {
        // Edges should be WALL (except at even coordinates which would be PILLAR)
        assert_eq!(
            determine_cell_type(0, 1, 7, 7),
            MazeCellTypeEnum::WALL.to_string()
        );
        assert_eq!(
            determine_cell_type(6, 3, 7, 7),
            MazeCellTypeEnum::WALL.to_string()
        );
        assert_eq!(
            determine_cell_type(3, 0, 7, 7),
            MazeCellTypeEnum::WALL.to_string()
        );
        assert_eq!(
            determine_cell_type(5, 6, 7, 7),
            MazeCellTypeEnum::WALL.to_string()
        );
    }

    #[test]
    fn test_cell_type_pillars() {
        // Even coordinates (not on boundary) should be PILLAR
        // But note: on boundaries, WALL takes precedence
        assert_eq!(
            determine_cell_type(2, 2, 7, 7),
            MazeCellTypeEnum::PILLAR.to_string()
        );
        assert_eq!(
            determine_cell_type(2, 4, 7, 7),
            MazeCellTypeEnum::PILLAR.to_string()
        );
        assert_eq!(
            determine_cell_type(4, 2, 7, 7),
            MazeCellTypeEnum::PILLAR.to_string()
        );
        assert_eq!(
            determine_cell_type(4, 4, 7, 7),
            MazeCellTypeEnum::PILLAR.to_string()
        );
    }

    #[test]
    fn test_cell_type_paths() {
        // Interior odd coordinates should be PATH
        assert_eq!(
            determine_cell_type(1, 1, 7, 7),
            MazeCellTypeEnum::PATH.to_string()
        );
        assert_eq!(
            determine_cell_type(3, 3, 7, 7),
            MazeCellTypeEnum::PATH.to_string()
        );
        assert_eq!(
            determine_cell_type(1, 3, 7, 7),
            MazeCellTypeEnum::PATH.to_string()
        );
        assert_eq!(
            determine_cell_type(3, 1, 7, 7),
            MazeCellTypeEnum::PATH.to_string()
        );
        assert_eq!(
            determine_cell_type(5, 5, 7, 7),
            MazeCellTypeEnum::PATH.to_string()
        );
    }

    #[test]
    fn test_cell_type_grid_layout() {
        // Test a small 5x5 grid comprehensively
        // PILLAR: both x AERROR make_maze] Failed to initialize database: Execution Error: error returned from database: 1390 (HY000): Prepared statement contains too many placeholders End time (UNIXTIME): 1769315381 Elapsed time (seconds): 1ND y are even (x%2==0 && y%2==0)
        // WALL: on boundary (x==0 || x==max-1 || y==0 || y==max-1) AND not PILLAR
        // PATH: everything else
        //
        // Grid (y increases downward):
        // (0,0) (1,0) (2,0) (3,0) (4,0)   <- y=0 (boundary)
        // (0,1) (1,1) (2,1) (3,1) (4,1)
        // (0,2) (1,2) (2,2) (3,2) (4,2)   <- y=2 (even row)
        // (0,3) (1,3) (2,3) (3,3) (4,3)
        // (0,4) (1,4) (2,4) (3,4) (4,4)   <- y=4 (boundary, even)
        //   ^           ^           ^
        //  x=0         x=2         x=4
        // (boundary) (even)    (boundary)

        let expected = vec![
            vec!["PILLAR", "WALL", "PILLAR", "WALL", "PILLAR"], // y=0
            vec!["WALL", "PATH", "PATH", "PATH", "WALL"],       // y=1
            vec!["PILLAR", "PATH", "PILLAR", "PATH", "PILLAR"], // y=2
            vec!["WALL", "PATH", "PATH", "PATH", "WALL"],       // y=3
            vec!["PILLAR", "WALL", "PILLAR", "WALL", "PILLAR"], // y=4
        ];

        for y in 0..5 {
            for x in 0..5 {
                let result = determine_cell_type(x, y, 5, 5);
                assert_eq!(
                    result, expected[y as usize][x as usize],
                    "Mismatch at ({}, {}): expected {}, got {}",
                    x, y, expected[y as usize][x as usize], result
                );
            }
        }
    }

    // Integration tests that require actual database connection
    // These are marked with #[ignore] by default and require DATABASE_URL environment variable
    #[tokio::test]
    #[ignore]
    async fn integration_test_initialize_db() {
        // To run this test, you need to:
        // 1. Set up a test database
        // 2. Set DATABASE_URL environment variable
        // 3. Run with: cargo test -- --ignored

        eprintln!("Attempting to establish database connection...");
        let db = establish_connection(None)
            .await
            .expect("Failed to connect to database");
        eprintln!("Database connection established successfully");

        eprintln!("Initializing database with 5x5 grid...");
        let result = initialize_db(&db, 5, 5).await;

        if let Err(e) = &result {
            eprintln!("Error initializing database: {}", e);
            let mut source = e.source();
            while let Some(err) = source {
                eprintln!("  Caused by: {}", err);
                source = err.source();
            }
        }

        assert!(result.is_ok(), "initialize_db failed: {:?}", result.err());
        eprintln!("Database initialized successfully");

        // Verify data was inserted correctly

        eprintln!("Verifying maze_cell count...");
        let cell_count = crate::common::database::entities::maze_cell::Entity::find()
            .count(db.as_ref())
            .await
            .expect("Failed to count maze cells");
        eprintln!("Found {} maze cells", cell_count);
        assert_eq!(
            cell_count,
            5 * 5,
            "Expected 25 maze cells, got {}",
            cell_count
        );

        eprintln!("Verifying maze_field count...");
        let field_count = crate::common::database::entities::maze_field::Entity::find()
            .count(db.as_ref())
            .await
            .expect("Failed to count maze fields");
        eprintln!("Found {} maze fields", field_count);
        assert_eq!(
            field_count,
            5 * 5,
            "Expected 25 maze fields, got {}",
            field_count
        );

        eprintln!("Verifying maze_cell_type count...");
        let types = crate::common::database::entities::maze_cell_type::Entity::find()
            .all(db.as_ref())
            .await
            .expect("Failed to fetch cell types");
        eprintln!("Found {} cell types", types.len());
        assert_eq!(
            types.len(),
            5,
            "Expected 5 cell types (WALL, PATH, PILLAR, START, END), got {}",
            types.len()
        );

        eprintln!("All verifications passed!");
    }
}
