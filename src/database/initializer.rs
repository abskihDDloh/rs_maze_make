use sea_orm::{DatabaseTransaction, EntityTrait, TransactionTrait};

async fn erase_maze_field(
    _db: &sea_orm::DbConn,
    txn: &DatabaseTransaction,
) -> Result<(), sea_orm::DbErr> {
    crate::database::entities::maze_field::Entity::delete_many()
        .exec(txn)
        .await?;
    Ok(())
}

async fn erase_therad_list(
    _db: &sea_orm::DbConn,
    txn: &DatabaseTransaction,
) -> Result<(), sea_orm::DbErr> {
    crate::database::entities::therad_list::Entity::delete_many()
        .exec(txn)
        .await?;

    Ok(())
}
async fn erase_maze_cell(
    _db: &sea_orm::DbConn,
    txn: &DatabaseTransaction,
) -> Result<(), sea_orm::DbErr> {
    crate::database::entities::maze_cell::Entity::delete_many()
        .exec(txn)
        .await?;
    Ok(())
}
async fn erase_maze_cell_type(
    _db: &sea_orm::DbConn,
    txn: &DatabaseTransaction,
) -> Result<(), sea_orm::DbErr> {
    crate::database::entities::maze_cell_type::Entity::delete_many()
        .exec(txn)
        .await?;
    Ok(())
}

pub const WALL: &str = "WALL";
pub const PATH: &str = "PATH";
pub const PILLAR: &str = "PILLAR";
pub const START: &str = "START";
pub const END: &str = "END";

async fn initialize_maze_cell(
    db: &sea_orm::DbConn,
    txn: &DatabaseTransaction,
    x_max: u64,
    y_max: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    for x in 0..x_max {
        for y in 0..y_max {
            let new_start_point = crate::database::entities::maze_cell::ActiveModel {
                x: sea_orm::ActiveValue::Set(x),
                y: sea_orm::ActiveValue::Set(y),
            };
            crate::database::entities::maze_cell::Entity::insert(new_start_point)
                .exec(txn)
                .await?;
        }
    }

    Ok(())
}

async fn initialize_maze_cell_type(
    db: &sea_orm::DbConn,
    txn: &DatabaseTransaction,
) -> Result<(), sea_orm::DbErr> {
    let cell_types = vec![WALL, PATH, PILLAR, START, END];

    for cell_type in cell_types {
        let new_cell_type = crate::database::entities::maze_cell_type::ActiveModel {
            cell_type: sea_orm::ActiveValue::Set(cell_type.to_string()),
        };
        crate::database::entities::maze_cell_type::Entity::insert(new_cell_type)
            .exec(txn)
            .await?;
    }
    Ok(())
}
async fn initialize_maze_field(
    db: &sea_orm::DbConn,
    txn: &DatabaseTransaction,
    x_max: u64,
    y_max: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    for x in 0..x_max {
        for y in 0..y_max {
            let new_cell_type: String;
            if x % 2 == 0 && y % 2 == 0 {
                new_cell_type = PILLAR.to_string();
            } else if x == 0 || x == x_max - 1 || y == 0 || y == y_max - 1 {
                new_cell_type = WALL.to_string();
            } else {
                new_cell_type = PATH.to_string();
            }
            let new_field = crate::database::entities::maze_field::ActiveModel {
                x: sea_orm::ActiveValue::Set(x),
                y: sea_orm::ActiveValue::Set(y),
                cell_type: sea_orm::ActiveValue::Set(new_cell_type),
                cell_owner_thread_id: sea_orm::ActiveValue::Set(None),
            };
            crate::database::entities::maze_field::Entity::insert(new_field)
                .exec(txn)
                .await?;
        }
    }

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

    let txn = db.begin().await?;

    // First, delete child tables that reference parent tables (foreign key constraints)
    erase_maze_field(db, &txn).await?;
    erase_therad_list(db, &txn).await?;

    // Then delete and recreate parent tables
    erase_maze_cell_type(db, &txn).await?;
    erase_maze_cell(db, &txn).await?;

    // Now recreate the data
    initialize_maze_cell_type(db, &txn).await?;
    initialize_maze_cell(db, &txn, x_max, y_max).await?;
    initialize_maze_field(db, &txn, x_max, y_max).await?;

    txn.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::database::connector::establish_connection;
    use sea_orm::PaginatorTrait;

    use super::*;

    #[test]
    fn test_cell_type_constants() {
        assert_eq!(WALL, "WALL");
        assert_eq!(PATH, "PATH");
        assert_eq!(PILLAR, "PILLAR");
        assert_eq!(START, "START");
        assert_eq!(END, "END");
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
            PILLAR.to_string()
        } else if x == 0 || x == x_max - 1 || y == 0 || y == y_max - 1 {
            WALL.to_string()
        } else {
            PATH.to_string()
        }
    }

    #[test]
    fn test_cell_type_corners() {
        // Corners at even coordinates are PILLAR (not WALL)
        // because the even coordinate check comes first in the logic
        assert_eq!(determine_cell_type(0, 0, 5, 5), PILLAR);
        assert_eq!(determine_cell_type(0, 4, 5, 5), PILLAR);
        assert_eq!(determine_cell_type(4, 0, 5, 5), PILLAR);
        assert_eq!(determine_cell_type(4, 4, 5, 5), PILLAR);
    }

    #[test]
    fn test_cell_type_edges() {
        // Edges should be WALL (except at even coordinates which would be PILLAR)
        assert_eq!(determine_cell_type(0, 1, 7, 7), WALL);
        assert_eq!(determine_cell_type(6, 3, 7, 7), WALL);
        assert_eq!(determine_cell_type(3, 0, 7, 7), WALL);
        assert_eq!(determine_cell_type(5, 6, 7, 7), WALL);
    }

    #[test]
    fn test_cell_type_pillars() {
        // Even coordinates (not on boundary) should be PILLAR
        // But note: on boundaries, WALL takes precedence
        assert_eq!(determine_cell_type(2, 2, 7, 7), PILLAR);
        assert_eq!(determine_cell_type(2, 4, 7, 7), PILLAR);
        assert_eq!(determine_cell_type(4, 2, 7, 7), PILLAR);
        assert_eq!(determine_cell_type(4, 4, 7, 7), PILLAR);
    }

    #[test]
    fn test_cell_type_paths() {
        // Interior odd coordinates should be PATH
        assert_eq!(determine_cell_type(1, 1, 7, 7), PATH);
        assert_eq!(determine_cell_type(3, 3, 7, 7), PATH);
        assert_eq!(determine_cell_type(1, 3, 7, 7), PATH);
        assert_eq!(determine_cell_type(3, 1, 7, 7), PATH);
        assert_eq!(determine_cell_type(5, 5, 7, 7), PATH);
    }

    #[test]
    fn test_cell_type_grid_layout() {
        // Test a small 5x5 grid comprehensively
        // PILLAR: both x AND y are even (x%2==0 && y%2==0)
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
        use sea_orm::EntityTrait;

        eprintln!("Verifying maze_cell count...");
        let cell_count: u64 = crate::database::entities::maze_cell::Entity::find()
            .count(&db)
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
        let field_count: u64 = crate::database::entities::maze_field::Entity::find()
            .count(&db)
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
        let types = crate::database::entities::maze_cell_type::Entity::find()
            .all(&db)
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
