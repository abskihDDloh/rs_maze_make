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
  `X` bigint(20) unsigned NOT NULL,
  `Y` bigint(20) unsigned NOT NULL,
  PRIMARY KEY (`X`,`Y`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;
/*!40101 SET character_set_client = @saved_cs_client */;

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
  `X` bigint(20) unsigned NOT NULL,
  `Y` bigint(20) unsigned NOT NULL,
  `CELL_TYPE` varchar(20) NOT NULL,
  `CELL_OWNER_THREAD_ID` bigint(20) unsigned DEFAULT NULL,
  PRIMARY KEY (`X`,`Y`),
  UNIQUE KEY `UNIQUE_CELLS` (`X`,`Y`,`CELL_OWNER_THREAD_ID`) USING BTREE,
  KEY `FK_TYPE` (`CELL_TYPE`),
  KEY `FK_OWNER` (`CELL_OWNER_THREAD_ID`),
  CONSTRAINT `FK_CELL` FOREIGN KEY (`X`, `Y`) REFERENCES `MAZE_CELL` (`X`, `Y`),
  CONSTRAINT `FK_OWNER` FOREIGN KEY (`CELL_OWNER_THREAD_ID`) REFERENCES `THERAD_LIST` (`ID`),
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
 1 AS `X`,
  1 AS `Y` */;
SET character_set_client = @saved_cs_client;

--
-- Temporary table structure for view `START_POINTS_VIEW`
--

DROP TABLE IF EXISTS `START_POINTS_VIEW`;
/*!50001 DROP VIEW IF EXISTS `START_POINTS_VIEW`*/;
SET @saved_cs_client     = @@character_set_client;
SET character_set_client = utf8mb4;
/*!50001 CREATE VIEW `START_POINTS_VIEW` AS SELECT
 1 AS `X`,
  1 AS `Y` */;
SET character_set_client = @saved_cs_client;

--
-- Table structure for table `THERAD_LIST`
--

DROP TABLE IF EXISTS `THERAD_LIST`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `THERAD_LIST` (
  `ID` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `THREAD_ID` varchar(80) NOT NULL,
  `CREATE_UNIXTIME` timestamp(6) NOT NULL DEFAULT '0000-00-00 00:00:00.000000',
  `START_X` bigint(20) unsigned NOT NULL,
  `START_Y` bigint(20) unsigned NOT NULL,
  PRIMARY KEY (`ID`),
  UNIQUE KEY `UNIQUE_THREAD_IDENTIFICATION_COLUMN` (`THREAD_ID`,`CREATE_UNIXTIME`),
  UNIQUE KEY `UNIQUE_ALL_COLUMN` (`ID`,`THREAD_ID`,`CREATE_UNIXTIME`,`START_X`,`START_Y`) USING BTREE,
  UNIQUE KEY `UNIQUE_START_POINT` (`START_X`,`START_Y`),
  CONSTRAINT `FK_THREAD_START_POINT` FOREIGN KEY (`START_X`, `START_Y`) REFERENCES `MAZE_CELL` (`X`, `Y`)
) ENGINE=InnoDB AUTO_INCREMENT=6 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;
/*!40101 SET character_set_client = @saved_cs_client */;
/*!50003 SET @saved_cs_client      = @@character_set_client */ ;
/*!50003 SET @saved_cs_results     = @@character_set_results */ ;
/*!50003 SET @saved_col_connection = @@collation_connection */ ;
/*!50003 SET character_set_client  = utf8mb4 */ ;
/*!50003 SET character_set_results = utf8mb4 */ ;
/*!50003 SET collation_connection  = utf8mb4_unicode_ci */ ;
/*!50003 SET @saved_sql_mode       = @@sql_mode */ ;
/*!50003 SET sql_mode              = 'STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION' */ ;
DELIMITER ;;
/*!50003 CREATE*/ /*!50017 DEFINER=`mazemake_u`@`localhost`*/ /*!50003 TRIGGER `CHECK_CELL_BEFORE_INSERT` BEFORE INSERT ON `THERAD_LIST` FOR EACH ROW BEGIN

  IF NOT EXISTS (

    SELECT 1 

    FROM START_POINTS_VIEW 

    WHERE X = NEW.START_X AND Y = NEW.START_Y

  ) THEN

    SIGNAL SQLSTATE '45000' 

    SET MESSAGE_TEXT = 'MUST SET START_POINT.';

  END IF;

END */;;
DELIMITER ;
/*!50003 SET sql_mode              = @saved_sql_mode */ ;
/*!50003 SET character_set_client  = @saved_cs_client */ ;
/*!50003 SET character_set_results = @saved_cs_results */ ;
/*!50003 SET collation_connection  = @saved_col_connection */ ;

--
-- Temporary table structure for view `UNUSED_START_POINTS_VIEW`
--

DROP TABLE IF EXISTS `UNUSED_START_POINTS_VIEW`;
/*!50001 DROP VIEW IF EXISTS `UNUSED_START_POINTS_VIEW`*/;
SET @saved_cs_client     = @@character_set_client;
SET character_set_client = utf8mb4;
/*!50001 CREATE VIEW `UNUSED_START_POINTS_VIEW` AS SELECT
 1 AS `X`,
  1 AS `Y` */;
SET character_set_client = @saved_cs_client;

--
-- Temporary table structure for view `USED_OUTSIDE_WALL_START_POINTS_VIEW`
--

DROP TABLE IF EXISTS `USED_OUTSIDE_WALL_START_POINTS_VIEW`;
/*!50001 DROP VIEW IF EXISTS `USED_OUTSIDE_WALL_START_POINTS_VIEW`*/;
SET @saved_cs_client     = @@character_set_client;
SET character_set_client = utf8mb4;
/*!50001 CREATE VIEW `USED_OUTSIDE_WALL_START_POINTS_VIEW` AS SELECT
 1 AS `X`,
  1 AS `Y`,
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
 1 AS `X`,
  1 AS `Y`,
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
CREATE DEFINER=`mazemake_u`@`localhost` PROCEDURE `ADD_NEW_THREAD`(IN `p_x` BIGINT UNSIGNED, IN `p_y` BIGINT UNSIGNED, IN `p_thread_id` VARCHAR(80), IN `p_create_unixtime` TIMESTAMP)
    SQL SECURITY INVOKER
BEGIN
    DECLARE v_id BIGINT UNSIGNED;
    DECLARE v_start_x BIGINT UNSIGNED;
    DECLARE v_start_y BIGINT UNSIGNED;
    
    -- エラーハンドラー（SQLエラーで終了）
    DECLARE EXIT HANDLER FOR SQLEXCEPTION
    BEGIN
        ROLLBACK;
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'Error occurred in ADD_NEW_THREAD procedure';
    END;
    
    START TRANSACTION;
    
    -- THERAD_LISTにINSERT
    INSERT INTO THERAD_LIST (THREAD_ID, CREATE_UNIXTIME, START_X, START_Y) 
    VALUES (p_thread_id, p_create_unixtime, p_x, p_y);
    
    -- INSERTした行のID, START_X, START_Yを取得（1行だけ想定）
    SELECT ID, START_X, START_Y INTO v_id, v_start_x, v_start_y
    FROM THERAD_LIST
    WHERE THREAD_ID = p_thread_id AND CREATE_UNIXTIME = p_create_unixtime
    LIMIT 1;
    
    -- 1行だけ取得できたか確認、そうでなければエラー
    IF ROW_COUNT() <> 1 THEN
        ROLLBACK;
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'Failed to retrieve inserted row from THERAD_LIST';
    END IF;
    
    -- MAZE_FIELDをUPDATE
    UPDATE MAZE_FIELD 
    SET CELL_OWNER_THREAD_ID = v_id 
    WHERE X = v_start_x AND Y = v_start_y;
    
    -- UPDATEが失敗したらエラー
    IF ROW_COUNT() = 0 THEN
        ROLLBACK;
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'Failed to update MAZE_FIELD';
    END IF;
    
    COMMIT;
END ;;
DELIMITER ;
/*!50003 SET sql_mode              = @saved_sql_mode */ ;
/*!50003 SET character_set_client  = @saved_cs_client */ ;
/*!50003 SET character_set_results = @saved_cs_results */ ;
/*!50003 SET collation_connection  = @saved_col_connection */ ;

--
-- Final view structure for view `OUTSIDE_WALL_START_POINTS_VIEW`
--

/*!50001 DROP VIEW IF EXISTS `OUTSIDE_WALL_START_POINTS_VIEW`*/;
/*!50001 SET @saved_cs_client          = @@character_set_client */;
/*!50001 SET @saved_cs_results         = @@character_set_results */;
/*!50001 SET @saved_col_connection     = @@collation_connection */;
/*!50001 SET character_set_client      = utf8mb4 */;
/*!50001 SET character_set_results     = utf8mb4 */;
/*!50001 SET collation_connection      = utf8mb4_general_ci */;
/*!50001 CREATE ALGORITHM=UNDEFINED */
/*!50013 DEFINER=`mazemake_u`@`localhost` SQL SECURITY DEFINER */
/*!50001 VIEW `OUTSIDE_WALL_START_POINTS_VIEW` AS select `sp`.`X` AS `X`,`sp`.`Y` AS `Y` from (`START_POINTS_VIEW` `sp` join (select max(`START_POINTS_VIEW`.`X`) AS `max_x`,min(`START_POINTS_VIEW`.`X`) AS `min_x`,max(`START_POINTS_VIEW`.`Y`) AS `max_y`,min(`START_POINTS_VIEW`.`Y`) AS `min_y` from `START_POINTS_VIEW`) `b`) where `sp`.`X` = `b`.`max_x` or `sp`.`Y` = `b`.`max_y` or `sp`.`X` = `b`.`min_x` or `sp`.`Y` = `b`.`min_y` */;
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
/*!50001 VIEW `START_POINTS_VIEW` AS select `mc`.`X` AS `X`,`mc`.`Y` AS `Y` from (`MAZE_CELL` `mc` join (select max(`MAZE_CELL`.`X`) AS `max_x`,min(`MAZE_CELL`.`X`) AS `min_x`,max(`MAZE_CELL`.`Y`) AS `max_y`,min(`MAZE_CELL`.`Y`) AS `min_y` from `MAZE_CELL`) `b`) where `mc`.`X` MOD 2 = 0 and `mc`.`Y` MOD 2 = 0 and (`mc`.`X` <> `b`.`max_x` or `mc`.`Y` <> `b`.`max_y`) and (`mc`.`X` <> `b`.`min_x` or `mc`.`Y` <> `b`.`min_y`) and (`mc`.`X` <> `b`.`max_x` or `mc`.`Y` <> `b`.`min_y`) and (`mc`.`X` <> `b`.`min_x` or `mc`.`Y` <> `b`.`max_y`) */;
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
/*!50013 DEFINER=`mazemake_u`@`localhost` SQL SECURITY DEFINER */
/*!50001 VIEW `UNUSED_START_POINTS_VIEW` AS select `MAZE_FIELD`.`X` AS `X`,`MAZE_FIELD`.`Y` AS `Y` from (`MAZE_FIELD` join `START_POINTS_VIEW` on(`MAZE_FIELD`.`X` = `START_POINTS_VIEW`.`X` and `MAZE_FIELD`.`Y` = `START_POINTS_VIEW`.`Y`)) where `MAZE_FIELD`.`CELL_OWNER_THREAD_ID` is null */;
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
/*!50013 DEFINER=`mazemake_u`@`localhost` SQL SECURITY DEFINER */
/*!50001 VIEW `USED_OUTSIDE_WALL_START_POINTS_VIEW` AS select `o`.`X` AS `X`,`o`.`Y` AS `Y`,`t`.`THREAD_ID` AS `THREAD_ID`,`t`.`CREATE_UNIXTIME` AS `CREATE_UNIXTIME` from (`OUTSIDE_WALL_START_POINTS_VIEW` `o` join `THERAD_LIST` `t` on(`t`.`START_X` = `o`.`X` and `t`.`START_Y` = `o`.`Y`)) */;
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
/*!50001 VIEW `USED_START_POINTS_VIEW` AS select `MAZE_FIELD`.`X` AS `X`,`MAZE_FIELD`.`Y` AS `Y`,`THERAD_LIST`.`THREAD_ID` AS `THREAD_ID`,`THERAD_LIST`.`CREATE_UNIXTIME` AS `CREATE_UNIXTIME` from ((`MAZE_FIELD` join `START_POINTS_VIEW` on(`MAZE_FIELD`.`X` = `START_POINTS_VIEW`.`X` and `MAZE_FIELD`.`Y` = `START_POINTS_VIEW`.`Y`)) join `THERAD_LIST` on(`MAZE_FIELD`.`CELL_OWNER_THREAD_ID` = `THERAD_LIST`.`ID`)) where `MAZE_FIELD`.`CELL_OWNER_THREAD_ID` is not null */;
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

-- Dump completed on 2026-01-02 16:58:11
