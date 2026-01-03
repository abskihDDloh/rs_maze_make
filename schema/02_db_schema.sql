/*M!999999\- enable the sandbox mode */ 
-- MariaDB dump 10.19  Distrib 10.5.29-MariaDB, for Linux (x86_64)
--
-- Host: localhost    Database: MAZEMAKE
-- ------------------------------------------------------
-- Server version	10.5.29-MariaDB

/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!40101 SET NAMES utf8mb4 */;
/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;

--
-- Table structure for table `MAZE_CELL`
--

DROP TABLE IF EXISTS `MAZE_CELL`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `MAZE_CELL` (
  `ID` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `X` bigint(20) unsigned NOT NULL,
  `Y` bigint(20) unsigned NOT NULL,
  PRIMARY KEY (`ID`),
  UNIQUE KEY `UNIQUE_CELL` (`X`,`Y`) USING BTREE,
  UNIQUE KEY `UNIQUE_ALL` (`ID`,`X`,`Y`)
) ENGINE=InnoDB AUTO_INCREMENT=1403 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Temporary table structure for view `MAZE_CELL_MIN_MAX_VIEW`
--

DROP TABLE IF EXISTS `MAZE_CELL_MIN_MAX_VIEW`;
/*!50001 DROP VIEW IF EXISTS `MAZE_CELL_MIN_MAX_VIEW`*/;
SET @saved_cs_client     = @@character_set_client;
SET character_set_client = utf8mb4;
/*!50001 CREATE VIEW `MAZE_CELL_MIN_MAX_VIEW` AS SELECT
 1 AS `max_x`,
  1 AS `min_x`,
  1 AS `max_y`,
  1 AS `min_y` */;
SET character_set_client = @saved_cs_client;

--
-- Temporary table structure for view `MAZE_CELL_STATUS_VIEW`
--

DROP TABLE IF EXISTS `MAZE_CELL_STATUS_VIEW`;
/*!50001 DROP VIEW IF EXISTS `MAZE_CELL_STATUS_VIEW`*/;
SET @saved_cs_client     = @@character_set_client;
SET character_set_client = utf8mb4;
/*!50001 CREATE VIEW `MAZE_CELL_STATUS_VIEW` AS SELECT
 1 AS `CELL_ID`,
  1 AS `X`,
  1 AS `Y`,
  1 AS `CELL_TYPE`,
  1 AS `CELL_OWNER_THREAD_ID` */;
SET character_set_client = @saved_cs_client;

--
-- Table structure for table `MAZE_CELL_TYPE`
--

DROP TABLE IF EXISTS `MAZE_CELL_TYPE`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `MAZE_CELL_TYPE` (
  `CELL_TYPE` varchar(20) NOT NULL,
  PRIMARY KEY (`CELL_TYPE`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `MAZE_FIELD`
--

DROP TABLE IF EXISTS `MAZE_FIELD`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `MAZE_FIELD` (
  `ID` bigint(20) unsigned NOT NULL,
  `CELL_TYPE` varchar(20) NOT NULL,
  `CELL_OWNER_THREAD_ID` bigint(20) unsigned DEFAULT NULL,
  PRIMARY KEY (`ID`),
  KEY `FK_TYPE` (`CELL_TYPE`),
  KEY `FK_OWNER` (`CELL_OWNER_THREAD_ID`),
  CONSTRAINT `FK_CELL` FOREIGN KEY (`ID`) REFERENCES `MAZE_CELL` (`ID`),
  CONSTRAINT `FK_OWNER` FOREIGN KEY (`CELL_OWNER_THREAD_ID`) REFERENCES `THREAD_LIST` (`ID`),
  CONSTRAINT `FK_TYPE` FOREIGN KEY (`CELL_TYPE`) REFERENCES `MAZE_CELL_TYPE` (`CELL_TYPE`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Temporary table structure for view `OUTSIDE_WALL_START_POINTS_VIEW`
--

DROP TABLE IF EXISTS `OUTSIDE_WALL_START_POINTS_VIEW`;
/*!50001 DROP VIEW IF EXISTS `OUTSIDE_WALL_START_POINTS_VIEW`*/;
SET @saved_cs_client     = @@character_set_client;
SET character_set_client = utf8mb4;
/*!50001 CREATE VIEW `OUTSIDE_WALL_START_POINTS_VIEW` AS SELECT
 1 AS `CELL_ID`,
  1 AS `X`,
  1 AS `Y`,
  1 AS `CELL_TYPE` */;
SET character_set_client = @saved_cs_client;

--
-- Temporary table structure for view `START_POINTS_VIEW`
--

DROP TABLE IF EXISTS `START_POINTS_VIEW`;
/*!50001 DROP VIEW IF EXISTS `START_POINTS_VIEW`*/;
SET @saved_cs_client     = @@character_set_client;
SET character_set_client = utf8mb4;
/*!50001 CREATE VIEW `START_POINTS_VIEW` AS SELECT
 1 AS `CELL_ID`,
  1 AS `X`,
  1 AS `Y`,
  1 AS `CELL_TYPE` */;
SET character_set_client = @saved_cs_client;

--
-- Table structure for table `THREAD_LIST`
--

DROP TABLE IF EXISTS `THREAD_LIST`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `THREAD_LIST` (
  `ID` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `THREAD_ID` varchar(80) NOT NULL,
  `CREATE_UNIXTIME` bigint(20) NOT NULL,
  `START_CELL` bigint(20) unsigned NOT NULL,
  PRIMARY KEY (`ID`),
  UNIQUE KEY `UNIQUE_THREAD_IDENTIFICATION_COLUMN` (`THREAD_ID`,`CREATE_UNIXTIME`),
  UNIQUE KEY `UNIQUE_ALL_COLUMN` (`ID`,`THREAD_ID`,`CREATE_UNIXTIME`) USING BTREE,
  UNIQUE KEY `UNIQUE_START_CELL` (`START_CELL`),
  CONSTRAINT `FK_START_CELL` FOREIGN KEY (`START_CELL`) REFERENCES `MAZE_CELL` (`ID`)
) ENGINE=InnoDB AUTO_INCREMENT=147 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Temporary table structure for view `UNUSED_START_POINTS_VIEW`
--

DROP TABLE IF EXISTS `UNUSED_START_POINTS_VIEW`;
/*!50001 DROP VIEW IF EXISTS `UNUSED_START_POINTS_VIEW`*/;
SET @saved_cs_client     = @@character_set_client;
SET character_set_client = utf8mb4;
/*!50001 CREATE VIEW `UNUSED_START_POINTS_VIEW` AS SELECT
 1 AS `CELL_ID`,
  1 AS `X`,
  1 AS `Y`,
  1 AS `CELL_TYPE` */;
SET character_set_client = @saved_cs_client;

--
-- Temporary table structure for view `USED_OUTSIDE_WALL_START_POINTS_VIEW`
--

DROP TABLE IF EXISTS `USED_OUTSIDE_WALL_START_POINTS_VIEW`;
/*!50001 DROP VIEW IF EXISTS `USED_OUTSIDE_WALL_START_POINTS_VIEW`*/;
SET @saved_cs_client     = @@character_set_client;
SET character_set_client = utf8mb4;
/*!50001 CREATE VIEW `USED_OUTSIDE_WALL_START_POINTS_VIEW` AS SELECT
 1 AS `CELL_ID`,
  1 AS `X`,
  1 AS `Y`,
  1 AS `CELL_TYPE`,
  1 AS `CELL_OWNER_THREAD_ID`,
  1 AS `THREAD_ID`,
  1 AS `CREATE_UNIXTIME` */;
SET character_set_client = @saved_cs_client;

--
-- Temporary table structure for view `USED_START_POINTS_VIEW`
--

DROP TABLE IF EXISTS `USED_START_POINTS_VIEW`;
/*!50001 DROP VIEW IF EXISTS `USED_START_POINTS_VIEW`*/;
SET @saved_cs_client     = @@character_set_client;
SET character_set_client = utf8mb4;
/*!50001 CREATE VIEW `USED_START_POINTS_VIEW` AS SELECT
 1 AS `CELL_ID`,
  1 AS `X`,
  1 AS `Y`,
  1 AS `CELL_TYPE`,
  1 AS `CELL_OWNER_THREAD_ID`,
  1 AS `THREAD_ID`,
  1 AS `CREATE_UNIXTIME` */;
SET character_set_client = @saved_cs_client;

--
-- Dumping routines for database 'MAZEMAKE'
--
/*!50003 SET @saved_sql_mode       = @@sql_mode */ ;
/*!50003 SET sql_mode              = 'STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION' */ ;
/*!50003 DROP PROCEDURE IF EXISTS `ADD_NEW_THREAD` */;
/*!50003 SET @saved_cs_client      = @@character_set_client */ ;
/*!50003 SET @saved_cs_results     = @@character_set_results */ ;
/*!50003 SET @saved_col_connection = @@collation_connection */ ;
/*!50003 SET character_set_client  = utf8mb4 */ ;
/*!50003 SET character_set_results = utf8mb4 */ ;
/*!50003 SET collation_connection  = utf8mb4_unicode_ci */ ;
DELIMITER ;;
CREATE DEFINER=`mazemake_u`@`localhost` PROCEDURE `ADD_NEW_THREAD`(IN `p_x` BIGINT UNSIGNED, IN `p_y` BIGINT UNSIGNED, IN `p_thread_id` VARCHAR(255), IN `p_create_unixtime` BIGINT)
    SQL SECURITY INVOKER
BEGIN
    DECLARE v_start_cell_id BIGINT UNSIGNED;
    DECLARE v_thread_row_id BIGINT UNSIGNED;
    DECLARE v_owner BIGINT UNSIGNED;
    DECLARE v_rows INT;
    DECLARE v_message_text VARCHAR(512);
    DECLARE EXIT HANDLER FOR SQLEXCEPTION
    BEGIN
        ROLLBACK;
        RESIGNAL;
    END;

    START TRANSACTION;

    
    SELECT CELL_ID INTO v_start_cell_id
      FROM START_POINTS_VIEW
     WHERE X = p_x AND Y = p_y
     LIMIT 1;

    SET v_rows = ROW_COUNT();
    IF v_rows != 1 THEN
        SET v_message_text = CONCAT('START_CELL not found or not unique (X=', p_x, ', Y=', p_y, ')');
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_message_text;
    END IF;

    
    INSERT INTO THREAD_LIST (THREAD_ID, CREATE_UNIXTIME, START_CELL)
    VALUES (p_thread_id, p_create_unixtime, v_start_cell_id);

    SET v_rows = ROW_COUNT();
    IF v_rows != 1 THEN
        SET v_message_text = CONCAT('Insert into THERAD_LIST failed (THREAD_ID=', p_thread_id, ', CREATE_UNIXTIME=', p_create_unixtime, ')');
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_message_text;
    END IF;

    SET v_thread_row_id = LAST_INSERT_ID();

    
    SELECT CELL_OWNER_THREAD_ID INTO v_owner
      FROM MAZE_FIELD
     WHERE ID = v_start_cell_id
     FOR UPDATE;

    SET v_rows = ROW_COUNT();
    IF v_rows != 1 THEN
        SET v_message_text = CONCAT('MAZE_FIELD row not found for the cell (CELL_ID=', v_start_cell_id, ')');
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_message_text;
    END IF;

    IF v_owner IS NOT NULL THEN
        SET v_message_text = CONCAT('CELL_OWNER_THREAD_ID is already set (CELL_ID=', v_start_cell_id, ')');
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_message_text;
    END IF;

    
    UPDATE MAZE_FIELD
       SET CELL_OWNER_THREAD_ID = v_thread_row_id
     WHERE ID = v_start_cell_id
       AND CELL_OWNER_THREAD_ID IS NULL;

    SET v_rows = ROW_COUNT();
    IF v_rows != 1 THEN
        SET v_message_text = CONCAT('Update MAZE_FIELD failed or CELL_OWNER_THREAD_ID already set (CELL_ID=', v_start_cell_id, ')');
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_message_text;
    END IF;

    COMMIT;
END ;;
DELIMITER ;
/*!50003 SET sql_mode              = @saved_sql_mode */ ;
/*!50003 SET character_set_client  = @saved_cs_client */ ;
/*!50003 SET character_set_results = @saved_cs_results */ ;
/*!50003 SET collation_connection  = @saved_col_connection */ ;
/*!50003 SET @saved_sql_mode       = @@sql_mode */ ;
/*!50003 SET sql_mode              = 'STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION' */ ;
/*!50003 DROP PROCEDURE IF EXISTS `GET_START_POINT` */;
/*!50003 SET @saved_cs_client      = @@character_set_client */ ;
/*!50003 SET @saved_cs_results     = @@character_set_results */ ;
/*!50003 SET @saved_col_connection = @@collation_connection */ ;
/*!50003 SET character_set_client  = utf8mb4 */ ;
/*!50003 SET character_set_results = utf8mb4 */ ;
/*!50003 SET collation_connection  = utf8mb4_unicode_ci */ ;
DELIMITER ;;
CREATE DEFINER=`mazemake_u`@`localhost` PROCEDURE `GET_START_POINT`(IN `p_x` BIGINT UNSIGNED, IN `p_y` BIGINT UNSIGNED, IN `p_thread_id` VARCHAR(255), IN `p_create_unixtime` BIGINT)
    SQL SECURITY INVOKER
BEGIN
    DECLARE v_start_cell_id BIGINT UNSIGNED;
    DECLARE v_cell_type VARCHAR(255);
    DECLARE v_thread_row_id BIGINT UNSIGNED;
    DECLARE v_owner BIGINT UNSIGNED;
    DECLARE v_field_type VARCHAR(255);
    DECLARE v_used_cell_owner BIGINT UNSIGNED DEFAULT NULL;
    DECLARE v_result_status VARCHAR(20);
    DECLARE v_message_text VARCHAR(512);
    DECLARE EXIT HANDLER FOR SQLEXCEPTION
    BEGIN
        ROLLBACK;
        RESIGNAL;
    END;

    START TRANSACTION;

    
    
    SELECT CELL_ID, CELL_TYPE
      INTO v_start_cell_id, v_cell_type
      FROM UNUSED_START_POINTS_VIEW
     WHERE X = p_x AND Y = p_y
     LIMIT 1;

    
    IF v_start_cell_id IS NULL THEN
        SELECT CELL_ID, CELL_TYPE, CELL_OWNER_THREAD_ID
          INTO v_start_cell_id, v_cell_type, v_used_cell_owner
          FROM USED_START_POINTS_VIEW
         WHERE X = p_x AND Y = p_y
           AND (THREAD_ID <> p_thread_id OR CREATE_UNIXTIME <> p_create_unixtime)
         LIMIT 1;
    END IF;

    IF v_start_cell_id IS NULL AND v_used_cell_owner IS NULL THEN
        SET v_message_text = CONCAT('START_CELL not found for X=', p_x, ', Y=', p_y);
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_message_text;
    END IF;


    IF v_used_cell_owner IS NOT NULL THEN
        SELECT COUNT(*)
          INTO v_owner
          FROM USED_OUTSIDE_WALL_START_POINTS_VIEW
         WHERE CELL_OWNER_THREAD_ID = v_used_cell_owner;

        IF v_owner = 0 THEN
            SET v_message_text = CONCAT('USED_START_POINTS_VIEW record not found in USED_OUTSIDE_WALL_START_POINTS_VIEW (CELL_OWNER_THREAD_ID=', v_used_cell_owner, ')');
            SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_message_text;
        END IF;
    END IF;

    
    SELECT ID
      INTO v_thread_row_id
      FROM THREAD_LIST
     WHERE THREAD_ID = p_thread_id
       AND CREATE_UNIXTIME = p_create_unixtime
     LIMIT 1;

    IF v_thread_row_id IS NULL THEN
        SET v_message_text = CONCAT('THREAD row not found for THREAD_ID=', p_thread_id, ', CREATE_UNIXTIME=', p_create_unixtime);
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_message_text;
    END IF;

    
    SELECT CELL_OWNER_THREAD_ID, CELL_TYPE
      INTO v_owner, v_field_type
      FROM MAZE_FIELD
     WHERE ID = v_start_cell_id
     FOR UPDATE;

    IF v_owner IS NOT NULL AND (v_used_cell_owner IS NULL OR v_owner <> v_used_cell_owner) THEN
        SET v_message_text = CONCAT('CELL_OWNER_THREAD_ID already set (CELL_ID=', v_start_cell_id, ', CELL_OWNER_THREAD_ID=', v_owner, ')');
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_message_text;
    END IF;

    IF v_field_type <> v_cell_type THEN
        SET v_message_text = CONCAT('CELL_TYPE mismatch (expected=', v_cell_type, ', actual=', v_field_type, ')');
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_message_text;
    END IF;

    
    IF v_used_cell_owner IS NOT NULL THEN
        
        SET v_result_status = 'NOT_UPDATED';
    ELSE
        
        UPDATE MAZE_FIELD
           SET CELL_OWNER_THREAD_ID = v_thread_row_id
         WHERE ID = v_start_cell_id
           AND CELL_OWNER_THREAD_ID IS NULL;

        IF ROW_COUNT() <> 1 THEN
            SET v_message_text = CONCAT('Update failed or row state changed (CELL_ID=', v_start_cell_id, ')');
            SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_message_text;
        END IF;

        SET v_result_status = 'UPDATED';
    END IF;

    COMMIT;

    SELECT v_result_status AS RESULT_STATUS;
END ;;
DELIMITER ;
/*!50003 SET sql_mode              = @saved_sql_mode */ ;
/*!50003 SET character_set_client  = @saved_cs_client */ ;
/*!50003 SET character_set_results = @saved_cs_results */ ;
/*!50003 SET collation_connection  = @saved_col_connection */ ;
/*!50003 SET @saved_sql_mode       = @@sql_mode */ ;
/*!50003 SET sql_mode              = 'STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION' */ ;
/*!50003 DROP PROCEDURE IF EXISTS `INSERT_NEW_CELL` */;
/*!50003 SET @saved_cs_client      = @@character_set_client */ ;
/*!50003 SET @saved_cs_results     = @@character_set_results */ ;
/*!50003 SET @saved_col_connection = @@collation_connection */ ;
/*!50003 SET character_set_client  = utf8mb4 */ ;
/*!50003 SET character_set_results = utf8mb4 */ ;
/*!50003 SET collation_connection  = utf8mb4_unicode_ci */ ;
DELIMITER ;;
CREATE DEFINER=`mazemake_u`@`localhost` PROCEDURE `INSERT_NEW_CELL`(IN `p_x` BIGINT UNSIGNED, IN `p_y` BIGINT UNSIGNED, IN `p_cell_type` VARCHAR(255))
    SQL SECURITY INVOKER
BEGIN
    DECLARE v_cell_id BIGINT UNSIGNED;
    DECLARE v_row_count INT;
    DECLARE EXIT HANDLER FOR SQLEXCEPTION
    BEGIN
        ROLLBACK;
        RESIGNAL;
    END;

    START TRANSACTION;

    
    INSERT INTO MAZE_CELL (X, Y)
    VALUES (p_x, p_y);

    
    SET v_row_count = ROW_COUNT();
    IF v_row_count != 1 THEN
        SIGNAL SQLSTATE '45000'
        SET MESSAGE_TEXT = 'Failed to insert into MAZE_CELL or unexpected row count';
    END IF;

    
    SET v_cell_id = LAST_INSERT_ID();

    
    INSERT INTO MAZE_FIELD (ID, CELL_TYPE)
    VALUES (v_cell_id, p_cell_type);

    
    SET v_row_count = ROW_COUNT();
    IF v_row_count != 1 THEN
        SIGNAL SQLSTATE '45000'
        SET MESSAGE_TEXT = 'Failed to insert into MAZE_FIELD or unexpected row count';
    END IF;

    COMMIT;
END ;;
DELIMITER ;
/*!50003 SET sql_mode              = @saved_sql_mode */ ;
/*!50003 SET character_set_client  = @saved_cs_client */ ;
/*!50003 SET character_set_results = @saved_cs_results */ ;
/*!50003 SET collation_connection  = @saved_col_connection */ ;
/*!50003 SET @saved_sql_mode       = @@sql_mode */ ;
/*!50003 SET sql_mode              = 'STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION' */ ;
/*!50003 DROP PROCEDURE IF EXISTS `PATH_TO_WALL` */;
/*!50003 SET @saved_cs_client      = @@character_set_client */ ;
/*!50003 SET @saved_cs_results     = @@character_set_results */ ;
/*!50003 SET @saved_col_connection = @@collation_connection */ ;
/*!50003 SET character_set_client  = utf8mb4 */ ;
/*!50003 SET character_set_results = utf8mb4 */ ;
/*!50003 SET collation_connection  = utf8mb4_unicode_ci */ ;
DELIMITER ;;
CREATE DEFINER=`mazemake_u`@`localhost` PROCEDURE `PATH_TO_WALL`(IN `p_x` BIGINT UNSIGNED, IN `p_y` BIGINT UNSIGNED, IN `p_thread_id` VARCHAR(255), IN `p_create_unixtime` BIGINT)
    SQL SECURITY INVOKER
BEGIN
    DECLARE v_target_cell_id BIGINT UNSIGNED;
    DECLARE v_thread_row_id BIGINT UNSIGNED;
    DECLARE v_cell_type VARCHAR(255);
    DECLARE v_owner BIGINT UNSIGNED;
    DECLARE v_rows INT;
    DECLARE v_message_text VARCHAR(512);
    DECLARE EXIT HANDLER FOR SQLEXCEPTION
    BEGIN
        ROLLBACK;
        RESIGNAL;
    END;

    START TRANSACTION;

    
    SELECT ID INTO v_target_cell_id
      FROM MAZE_CELL
     WHERE X = p_x AND Y = p_y
     LIMIT 1;
    SET v_rows = ROW_COUNT();
    IF v_rows != 1 THEN
        SET v_message_text = CONCAT('START_CELL not found or not unique (X=', p_x, ', Y=', p_y, ')');
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_message_text;
    END IF;
    SELECT ID INTO v_thread_row_id
      FROM THREAD_LIST
     WHERE THREAD_ID = p_thread_id
       AND CREATE_UNIXTIME = p_create_unixtime
     LIMIT 1;
    SET v_rows = ROW_COUNT();
    IF v_rows != 1 THEN
        SET v_message_text = CONCAT('THREAD row not found or not unique (THREAD_ID=', p_thread_id, ', CREATE_UNIXTIME=', p_create_unixtime, ')');
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_message_text;
    END IF;

    
    SELECT CELL_OWNER_THREAD_ID, CELL_TYPE
      INTO v_owner, v_cell_type
      FROM MAZE_FIELD
     WHERE ID = v_target_cell_id
     FOR UPDATE;
    SET v_rows = ROW_COUNT();
    IF v_rows != 1 THEN
        SET v_message_text = CONCAT('MAZE_FIELD row not found (CELL_ID=', v_target_cell_id, ')');
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_message_text;
    END IF;
    IF v_owner IS NOT NULL THEN
        SET v_message_text = CONCAT('CELL_OWNER_THREAD_ID is already set (CELL_ID=', v_target_cell_id, ')');
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_message_text;
    END IF;
    IF v_cell_type <> 'PATH' THEN
        SET v_message_text = CONCAT('CELL_TYPE is not PATH (CELL_ID=', v_target_cell_id, ', CELL_TYPE=', v_cell_type, ')');
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_message_text;
    END IF;

    
    UPDATE MAZE_FIELD
       SET CELL_OWNER_THREAD_ID = v_thread_row_id,
           CELL_TYPE = 'WALL'
     WHERE ID = v_target_cell_id
       AND CELL_OWNER_THREAD_ID IS NULL
       AND CELL_TYPE = 'PATH';
    SET v_rows = ROW_COUNT();
    IF v_rows != 1 THEN
        SET v_message_text = CONCAT('Update failed or row state changed (CELL_ID=', v_target_cell_id, ')');
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_message_text;
    END IF;

    COMMIT;
END ;;
DELIMITER ;
/*!50003 SET sql_mode              = @saved_sql_mode */ ;
/*!50003 SET character_set_client  = @saved_cs_client */ ;
/*!50003 SET character_set_results = @saved_cs_results */ ;
/*!50003 SET collation_connection  = @saved_col_connection */ ;

--
-- Final view structure for view `MAZE_CELL_MIN_MAX_VIEW`
--

/*!50001 DROP VIEW IF EXISTS `MAZE_CELL_MIN_MAX_VIEW`*/;
/*!50001 SET @saved_cs_client          = @@character_set_client */;
/*!50001 SET @saved_cs_results         = @@character_set_results */;
/*!50001 SET @saved_col_connection     = @@collation_connection */;
/*!50001 SET character_set_client      = utf8mb4 */;
/*!50001 SET character_set_results     = utf8mb4 */;
/*!50001 SET collation_connection      = utf8mb4_unicode_ci */;
/*!50001 CREATE ALGORITHM=UNDEFINED */
/*!50013 DEFINER=`mazemake_u`@`localhost` SQL SECURITY INVOKER */
/*!50001 VIEW `MAZE_CELL_MIN_MAX_VIEW` AS select max(`MAZE_CELL`.`X`) AS `max_x`,min(`MAZE_CELL`.`X`) AS `min_x`,max(`MAZE_CELL`.`Y`) AS `max_y`,min(`MAZE_CELL`.`Y`) AS `min_y` from `MAZE_CELL` */;
/*!50001 SET character_set_client      = @saved_cs_client */;
/*!50001 SET character_set_results     = @saved_cs_results */;
/*!50001 SET collation_connection      = @saved_col_connection */;

--
-- Final view structure for view `MAZE_CELL_STATUS_VIEW`
--

/*!50001 DROP VIEW IF EXISTS `MAZE_CELL_STATUS_VIEW`*/;
/*!50001 SET @saved_cs_client          = @@character_set_client */;
/*!50001 SET @saved_cs_results         = @@character_set_results */;
/*!50001 SET @saved_col_connection     = @@collation_connection */;
/*!50001 SET character_set_client      = utf8mb4 */;
/*!50001 SET character_set_results     = utf8mb4 */;
/*!50001 SET collation_connection      = utf8mb4_unicode_ci */;
/*!50001 CREATE ALGORITHM=UNDEFINED */
/*!50013 DEFINER=`mazemake_u`@`localhost` SQL SECURITY INVOKER */
/*!50001 VIEW `MAZE_CELL_STATUS_VIEW` AS select `MAZE_CELL`.`ID` AS `CELL_ID`,`MAZE_CELL`.`X` AS `X`,`MAZE_CELL`.`Y` AS `Y`,`MAZE_FIELD`.`CELL_TYPE` AS `CELL_TYPE`,`MAZE_FIELD`.`CELL_OWNER_THREAD_ID` AS `CELL_OWNER_THREAD_ID` from (`MAZE_FIELD` join `MAZE_CELL` on(`MAZE_CELL`.`ID` = `MAZE_FIELD`.`ID`)) */;
/*!50001 SET character_set_client      = @saved_cs_client */;
/*!50001 SET character_set_results     = @saved_cs_results */;
/*!50001 SET collation_connection      = @saved_col_connection */;

--
-- Final view structure for view `OUTSIDE_WALL_START_POINTS_VIEW`
--

/*!50001 DROP VIEW IF EXISTS `OUTSIDE_WALL_START_POINTS_VIEW`*/;
/*!50001 SET @saved_cs_client          = @@character_set_client */;
/*!50001 SET @saved_cs_results         = @@character_set_results */;
/*!50001 SET @saved_col_connection     = @@collation_connection */;
/*!50001 SET character_set_client      = utf8mb4 */;
/*!50001 SET character_set_results     = utf8mb4 */;
/*!50001 SET collation_connection      = utf8mb4_unicode_ci */;
/*!50001 CREATE ALGORITHM=UNDEFINED */
/*!50013 DEFINER=`mazemake_u`@`localhost` SQL SECURITY INVOKER */
/*!50001 VIEW `OUTSIDE_WALL_START_POINTS_VIEW` AS select `sp`.`CELL_ID` AS `CELL_ID`,`sp`.`X` AS `X`,`sp`.`Y` AS `Y`,`sp`.`CELL_TYPE` AS `CELL_TYPE` from (`START_POINTS_VIEW` `sp` join (select max(`START_POINTS_VIEW`.`X`) AS `max_x`,min(`START_POINTS_VIEW`.`X`) AS `min_x`,max(`START_POINTS_VIEW`.`Y`) AS `max_y`,min(`START_POINTS_VIEW`.`Y`) AS `min_y` from `START_POINTS_VIEW`) `b`) where `sp`.`X` = `b`.`max_x` or `sp`.`Y` = `b`.`max_y` or `sp`.`X` = `b`.`min_x` or `sp`.`Y` = `b`.`min_y` */;
/*!50001 SET character_set_client      = @saved_cs_client */;
/*!50001 SET character_set_results     = @saved_cs_results */;
/*!50001 SET collation_connection      = @saved_col_connection */;

--
-- Final view structure for view `START_POINTS_VIEW`
--

/*!50001 DROP VIEW IF EXISTS `START_POINTS_VIEW`*/;
/*!50001 SET @saved_cs_client          = @@character_set_client */;
/*!50001 SET @saved_cs_results         = @@character_set_results */;
/*!50001 SET @saved_col_connection     = @@collation_connection */;
/*!50001 SET character_set_client      = utf8mb4 */;
/*!50001 SET character_set_results     = utf8mb4 */;
/*!50001 SET collation_connection      = utf8mb4_unicode_ci */;
/*!50001 CREATE ALGORITHM=UNDEFINED */
/*!50013 DEFINER=`mazemake_u`@`localhost` SQL SECURITY INVOKER */
/*!50001 VIEW `START_POINTS_VIEW` AS select `mc`.`CELL_ID` AS `CELL_ID`,`mc`.`X` AS `X`,`mc`.`Y` AS `Y`,`mc`.`CELL_TYPE` AS `CELL_TYPE` from (`MAZE_CELL_STATUS_VIEW` `mc` join (select `MAZE_CELL_MIN_MAX_VIEW`.`max_x` AS `max_x`,`MAZE_CELL_MIN_MAX_VIEW`.`min_x` AS `min_x`,`MAZE_CELL_MIN_MAX_VIEW`.`max_y` AS `max_y`,`MAZE_CELL_MIN_MAX_VIEW`.`min_y` AS `min_y` from `MAZE_CELL_MIN_MAX_VIEW`) `b`) where `mc`.`X` MOD 2 = 0 and `mc`.`Y` MOD 2 = 0 and (`mc`.`X` <> `b`.`max_x` or `mc`.`Y` <> `b`.`max_y`) and (`mc`.`X` <> `b`.`min_x` or `mc`.`Y` <> `b`.`min_y`) and (`mc`.`X` <> `b`.`max_x` or `mc`.`Y` <> `b`.`min_y`) and (`mc`.`X` <> `b`.`min_x` or `mc`.`Y` <> `b`.`max_y`) */;
/*!50001 SET character_set_client      = @saved_cs_client */;
/*!50001 SET character_set_results     = @saved_cs_results */;
/*!50001 SET collation_connection      = @saved_col_connection */;

--
-- Final view structure for view `UNUSED_START_POINTS_VIEW`
--

/*!50001 DROP VIEW IF EXISTS `UNUSED_START_POINTS_VIEW`*/;
/*!50001 SET @saved_cs_client          = @@character_set_client */;
/*!50001 SET @saved_cs_results         = @@character_set_results */;
/*!50001 SET @saved_col_connection     = @@collation_connection */;
/*!50001 SET character_set_client      = utf8mb4 */;
/*!50001 SET character_set_results     = utf8mb4 */;
/*!50001 SET collation_connection      = utf8mb4_unicode_ci */;
/*!50001 CREATE ALGORITHM=UNDEFINED */
/*!50013 DEFINER=`mazemake_u`@`localhost` SQL SECURITY INVOKER */
/*!50001 VIEW `UNUSED_START_POINTS_VIEW` AS select `START_POINTS_VIEW`.`CELL_ID` AS `CELL_ID`,`START_POINTS_VIEW`.`X` AS `X`,`START_POINTS_VIEW`.`Y` AS `Y`,`START_POINTS_VIEW`.`CELL_TYPE` AS `CELL_TYPE` from (`MAZE_FIELD` join `START_POINTS_VIEW` on(`MAZE_FIELD`.`ID` = `START_POINTS_VIEW`.`CELL_ID`)) where `MAZE_FIELD`.`CELL_OWNER_THREAD_ID` is null */;
/*!50001 SET character_set_client      = @saved_cs_client */;
/*!50001 SET character_set_results     = @saved_cs_results */;
/*!50001 SET collation_connection      = @saved_col_connection */;

--
-- Final view structure for view `USED_OUTSIDE_WALL_START_POINTS_VIEW`
--

/*!50001 DROP VIEW IF EXISTS `USED_OUTSIDE_WALL_START_POINTS_VIEW`*/;
/*!50001 SET @saved_cs_client          = @@character_set_client */;
/*!50001 SET @saved_cs_results         = @@character_set_results */;
/*!50001 SET @saved_col_connection     = @@collation_connection */;
/*!50001 SET character_set_client      = utf8mb4 */;
/*!50001 SET character_set_results     = utf8mb4 */;
/*!50001 SET collation_connection      = utf8mb4_unicode_ci */;
/*!50001 CREATE ALGORITHM=UNDEFINED */
/*!50013 DEFINER=`mazemake_u`@`localhost` SQL SECURITY INVOKER */
/*!50001 VIEW `USED_OUTSIDE_WALL_START_POINTS_VIEW` AS select `USED_START_POINTS_VIEW`.`CELL_ID` AS `CELL_ID`,`USED_START_POINTS_VIEW`.`X` AS `X`,`USED_START_POINTS_VIEW`.`Y` AS `Y`,`USED_START_POINTS_VIEW`.`CELL_TYPE` AS `CELL_TYPE`,`USED_START_POINTS_VIEW`.`CELL_OWNER_THREAD_ID` AS `CELL_OWNER_THREAD_ID`,`USED_START_POINTS_VIEW`.`THREAD_ID` AS `THREAD_ID`,`USED_START_POINTS_VIEW`.`CREATE_UNIXTIME` AS `CREATE_UNIXTIME` from (`OUTSIDE_WALL_START_POINTS_VIEW` join `USED_START_POINTS_VIEW` on(`OUTSIDE_WALL_START_POINTS_VIEW`.`CELL_ID` = `USED_START_POINTS_VIEW`.`CELL_ID`)) */;
/*!50001 SET character_set_client      = @saved_cs_client */;
/*!50001 SET character_set_results     = @saved_cs_results */;
/*!50001 SET collation_connection      = @saved_col_connection */;

--
-- Final view structure for view `USED_START_POINTS_VIEW`
--

/*!50001 DROP VIEW IF EXISTS `USED_START_POINTS_VIEW`*/;
/*!50001 SET @saved_cs_client          = @@character_set_client */;
/*!50001 SET @saved_cs_results         = @@character_set_results */;
/*!50001 SET @saved_col_connection     = @@collation_connection */;
/*!50001 SET character_set_client      = utf8mb4 */;
/*!50001 SET character_set_results     = utf8mb4 */;
/*!50001 SET collation_connection      = utf8mb4_unicode_ci */;
/*!50001 CREATE ALGORITHM=UNDEFINED */
/*!50013 DEFINER=`mazemake_u`@`localhost` SQL SECURITY INVOKER */
/*!50001 VIEW `USED_START_POINTS_VIEW` AS select `spv`.`CELL_ID` AS `CELL_ID`,`spv`.`X` AS `X`,`spv`.`Y` AS `Y`,`spv`.`CELL_TYPE` AS `CELL_TYPE`,`mf`.`CELL_OWNER_THREAD_ID` AS `CELL_OWNER_THREAD_ID`,`tl`.`THREAD_ID` AS `THREAD_ID`,`tl`.`CREATE_UNIXTIME` AS `CREATE_UNIXTIME` from (((select `MAZE_FIELD`.`ID` AS `ID`,`MAZE_FIELD`.`CELL_TYPE` AS `CELL_TYPE`,`MAZE_FIELD`.`CELL_OWNER_THREAD_ID` AS `CELL_OWNER_THREAD_ID` from `MAZE_FIELD` where `MAZE_FIELD`.`CELL_OWNER_THREAD_ID` is not null) `mf` join `START_POINTS_VIEW` `spv` on(`mf`.`ID` = `spv`.`CELL_ID`)) join `THREAD_LIST` `tl` on(`tl`.`ID` = `mf`.`CELL_OWNER_THREAD_ID`)) */;
/*!50001 SET character_set_client      = @saved_cs_client */;
/*!50001 SET character_set_results     = @saved_cs_results */;
/*!50001 SET collation_connection      = @saved_col_connection */;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

-- Dump completed on 2026-01-03 16:16:57
