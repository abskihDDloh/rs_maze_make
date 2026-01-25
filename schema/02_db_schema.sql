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
) ENGINE=InnoDB AUTO_INCREMENT=7639981 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;
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
-- Temporary table structure for view `MAZE_CELL_OWNER_VIEW`
--

DROP TABLE IF EXISTS `MAZE_CELL_OWNER_VIEW`;
/*!50001 DROP VIEW IF EXISTS `MAZE_CELL_OWNER_VIEW`*/;
SET @saved_cs_client     = @@character_set_client;
SET character_set_client = utf8mb4;
/*!50001 CREATE VIEW `MAZE_CELL_OWNER_VIEW` AS SELECT
 1 AS `CELL_ID`,
  1 AS `X`,
  1 AS `Y`,
  1 AS `CELL_TYPE`,
  1 AS `CELL_OWNER_THREAD_ID`,
  1 AS `THREAD_ID`,
  1 AS `CREATE_UNIXTIME`,
  1 AS `OUTSIDE_WALL_CONNECT_TYPE` */;
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
  KEY `idx_cell_type_owner` (`CELL_TYPE`,`CELL_OWNER_THREAD_ID`),
  CONSTRAINT `FK_CELL` FOREIGN KEY (`ID`) REFERENCES `MAZE_CELL` (`ID`),
  CONSTRAINT `FK_OWNER` FOREIGN KEY (`CELL_OWNER_THREAD_ID`) REFERENCES `THREAD_LIST` (`ID`),
  CONSTRAINT `FK_TYPE` FOREIGN KEY (`CELL_TYPE`) REFERENCES `MAZE_CELL_TYPE` (`CELL_TYPE`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `OUTSIDE_WALL_CONNECT_TYPE`
--

DROP TABLE IF EXISTS `OUTSIDE_WALL_CONNECT_TYPE`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `OUTSIDE_WALL_CONNECT_TYPE` (
  `TYPE` varchar(20) NOT NULL DEFAULT 'NOT_CONNECT',
  PRIMARY KEY (`TYPE`)
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
-- Temporary table structure for view `THREAD_FIRST_CELLS_LIST_VIEW`
--

DROP TABLE IF EXISTS `THREAD_FIRST_CELLS_LIST_VIEW`;
/*!50001 DROP VIEW IF EXISTS `THREAD_FIRST_CELLS_LIST_VIEW`*/;
SET @saved_cs_client     = @@character_set_client;
SET character_set_client = utf8mb4;
/*!50001 CREATE VIEW `THREAD_FIRST_CELLS_LIST_VIEW` AS SELECT
 1 AS `TID`,
  1 AS `THREAD_ID`,
  1 AS `CREATE_UNIXTIME`,
  1 AS `START_CELL_ID`,
  1 AS `START_X`,
  1 AS `START_Y`,
  1 AS `OUTSIDE_WALL_CONNECT_TYPE` */;
SET character_set_client = @saved_cs_client;

--
-- Temporary table structure for view `THREAD_FROM_OUTSIDE_WALL_VIEW`
--

DROP TABLE IF EXISTS `THREAD_FROM_OUTSIDE_WALL_VIEW`;
/*!50001 DROP VIEW IF EXISTS `THREAD_FROM_OUTSIDE_WALL_VIEW`*/;
SET @saved_cs_client     = @@character_set_client;
SET character_set_client = utf8mb4;
/*!50001 CREATE VIEW `THREAD_FROM_OUTSIDE_WALL_VIEW` AS SELECT
 1 AS `TID`,
  1 AS `THREAD_ID`,
  1 AS `CREATE_UNIXTIME`,
  1 AS `START_CELL_ID`,
  1 AS `START_X`,
  1 AS `START_Y`,
  1 AS `OUTSIDE_WALL_CONNECT_TYPE`,
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
  `OUTSIDE_WALL_CONNECT_TYPE` varchar(20) NOT NULL,
  PRIMARY KEY (`ID`),
  UNIQUE KEY `UNIQUE_THREAD_IDENTIFICATION_COLUMN` (`THREAD_ID`,`CREATE_UNIXTIME`),
  UNIQUE KEY `UNIQUE_START_CELL` (`START_CELL`),
  UNIQUE KEY `UNIQUE_ALL_COLUMN` (`ID`,`THREAD_ID`,`CREATE_UNIXTIME`,`OUTSIDE_WALL_CONNECT_TYPE`) USING BTREE,
  KEY `FK_OUTSIDE_WALL_CONNECT` (`OUTSIDE_WALL_CONNECT_TYPE`),
  CONSTRAINT `FK_OUTSIDE_WALL_CONNECT` FOREIGN KEY (`OUTSIDE_WALL_CONNECT_TYPE`) REFERENCES `OUTSIDE_WALL_CONNECT_TYPE` (`TYPE`),
  CONSTRAINT `FK_START_CELL` FOREIGN KEY (`START_CELL`) REFERENCES `MAZE_CELL` (`ID`)
) ENGINE=InnoDB AUTO_INCREMENT=7424 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;
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
  1 AS `CREATE_UNIXTIME`,
  1 AS `OUTSIDE_WALL_CONNECT_TYPE` */;
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
  1 AS `CREATE_UNIXTIME`,
  1 AS `OUTSIDE_WALL_CONNECT_TYPE` */;
SET character_set_client = @saved_cs_client;

--
-- Dumping routines for database 'MAZEMAKE'
--

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
-- Final view structure for view `MAZE_CELL_OWNER_VIEW`
--

/*!50001 DROP VIEW IF EXISTS `MAZE_CELL_OWNER_VIEW`*/;
/*!50001 SET @saved_cs_client          = @@character_set_client */;
/*!50001 SET @saved_cs_results         = @@character_set_results */;
/*!50001 SET @saved_col_connection     = @@collation_connection */;
/*!50001 SET character_set_client      = utf8mb4 */;
/*!50001 SET character_set_results     = utf8mb4 */;
/*!50001 SET collation_connection      = utf8mb4_unicode_ci */;
/*!50001 CREATE ALGORITHM=UNDEFINED */
/*!50013 DEFINER=`mazemake_u`@`localhost` SQL SECURITY INVOKER */
/*!50001 VIEW `MAZE_CELL_OWNER_VIEW` AS select `MAZE_CELL_STATUS_VIEW`.`CELL_ID` AS `CELL_ID`,`MAZE_CELL_STATUS_VIEW`.`X` AS `X`,`MAZE_CELL_STATUS_VIEW`.`Y` AS `Y`,`MAZE_CELL_STATUS_VIEW`.`CELL_TYPE` AS `CELL_TYPE`,`MAZE_CELL_STATUS_VIEW`.`CELL_OWNER_THREAD_ID` AS `CELL_OWNER_THREAD_ID`,`THREAD_LIST`.`THREAD_ID` AS `THREAD_ID`,`THREAD_LIST`.`CREATE_UNIXTIME` AS `CREATE_UNIXTIME`,`THREAD_LIST`.`OUTSIDE_WALL_CONNECT_TYPE` AS `OUTSIDE_WALL_CONNECT_TYPE` from (`MAZE_CELL_STATUS_VIEW` join `THREAD_LIST` on(`MAZE_CELL_STATUS_VIEW`.`CELL_OWNER_THREAD_ID` = `THREAD_LIST`.`ID`)) */;
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
-- Final view structure for view `THREAD_FIRST_CELLS_LIST_VIEW`
--

/*!50001 DROP VIEW IF EXISTS `THREAD_FIRST_CELLS_LIST_VIEW`*/;
/*!50001 SET @saved_cs_client          = @@character_set_client */;
/*!50001 SET @saved_cs_results         = @@character_set_results */;
/*!50001 SET @saved_col_connection     = @@collation_connection */;
/*!50001 SET character_set_client      = utf8mb4 */;
/*!50001 SET character_set_results     = utf8mb4 */;
/*!50001 SET collation_connection      = utf8mb4_unicode_ci */;
/*!50001 CREATE ALGORITHM=UNDEFINED */
/*!50013 DEFINER=`mazemake_u`@`localhost` SQL SECURITY INVOKER */
/*!50001 VIEW `THREAD_FIRST_CELLS_LIST_VIEW` AS select `THREAD_LIST`.`ID` AS `TID`,`THREAD_LIST`.`THREAD_ID` AS `THREAD_ID`,`THREAD_LIST`.`CREATE_UNIXTIME` AS `CREATE_UNIXTIME`,`THREAD_LIST`.`START_CELL` AS `START_CELL_ID`,`MAZE_CELL`.`X` AS `START_X`,`MAZE_CELL`.`Y` AS `START_Y`,`THREAD_LIST`.`OUTSIDE_WALL_CONNECT_TYPE` AS `OUTSIDE_WALL_CONNECT_TYPE` from (`THREAD_LIST` join `MAZE_CELL` on(`THREAD_LIST`.`START_CELL` = `MAZE_CELL`.`ID`)) */;
/*!50001 SET character_set_client      = @saved_cs_client */;
/*!50001 SET character_set_results     = @saved_cs_results */;
/*!50001 SET collation_connection      = @saved_col_connection */;

--
-- Final view structure for view `THREAD_FROM_OUTSIDE_WALL_VIEW`
--

/*!50001 DROP VIEW IF EXISTS `THREAD_FROM_OUTSIDE_WALL_VIEW`*/;
/*!50001 SET @saved_cs_client          = @@character_set_client */;
/*!50001 SET @saved_cs_results         = @@character_set_results */;
/*!50001 SET @saved_col_connection     = @@collation_connection */;
/*!50001 SET character_set_client      = utf8mb4 */;
/*!50001 SET character_set_results     = utf8mb4 */;
/*!50001 SET collation_connection      = utf8mb4_unicode_ci */;
/*!50001 CREATE ALGORITHM=UNDEFINED */
/*!50013 DEFINER=`mazemake_u`@`localhost` SQL SECURITY INVOKER */
/*!50001 VIEW `THREAD_FROM_OUTSIDE_WALL_VIEW` AS select `THREAD_FIRST_CELLS_LIST_VIEW`.`TID` AS `TID`,`THREAD_FIRST_CELLS_LIST_VIEW`.`THREAD_ID` AS `THREAD_ID`,`THREAD_FIRST_CELLS_LIST_VIEW`.`CREATE_UNIXTIME` AS `CREATE_UNIXTIME`,`THREAD_FIRST_CELLS_LIST_VIEW`.`START_CELL_ID` AS `START_CELL_ID`,`THREAD_FIRST_CELLS_LIST_VIEW`.`START_X` AS `START_X`,`THREAD_FIRST_CELLS_LIST_VIEW`.`START_Y` AS `START_Y`,`THREAD_FIRST_CELLS_LIST_VIEW`.`OUTSIDE_WALL_CONNECT_TYPE` AS `OUTSIDE_WALL_CONNECT_TYPE`,`OUTSIDE_WALL_START_POINTS_VIEW`.`CELL_TYPE` AS `CELL_TYPE` from (`THREAD_FIRST_CELLS_LIST_VIEW` join `OUTSIDE_WALL_START_POINTS_VIEW` on(`THREAD_FIRST_CELLS_LIST_VIEW`.`START_CELL_ID` = `OUTSIDE_WALL_START_POINTS_VIEW`.`CELL_ID`)) */;
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
/*!50001 VIEW `USED_OUTSIDE_WALL_START_POINTS_VIEW` AS select `USED_START_POINTS_VIEW`.`CELL_ID` AS `CELL_ID`,`USED_START_POINTS_VIEW`.`X` AS `X`,`USED_START_POINTS_VIEW`.`Y` AS `Y`,`USED_START_POINTS_VIEW`.`CELL_TYPE` AS `CELL_TYPE`,`USED_START_POINTS_VIEW`.`CELL_OWNER_THREAD_ID` AS `CELL_OWNER_THREAD_ID`,`USED_START_POINTS_VIEW`.`THREAD_ID` AS `THREAD_ID`,`USED_START_POINTS_VIEW`.`CREATE_UNIXTIME` AS `CREATE_UNIXTIME`,`USED_START_POINTS_VIEW`.`OUTSIDE_WALL_CONNECT_TYPE` AS `OUTSIDE_WALL_CONNECT_TYPE` from (`OUTSIDE_WALL_START_POINTS_VIEW` join `USED_START_POINTS_VIEW` on(`OUTSIDE_WALL_START_POINTS_VIEW`.`CELL_ID` = `USED_START_POINTS_VIEW`.`CELL_ID`)) */;
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
/*!50001 VIEW `USED_START_POINTS_VIEW` AS select `spv`.`CELL_ID` AS `CELL_ID`,`spv`.`X` AS `X`,`spv`.`Y` AS `Y`,`spv`.`CELL_TYPE` AS `CELL_TYPE`,`mf`.`CELL_OWNER_THREAD_ID` AS `CELL_OWNER_THREAD_ID`,`tl`.`THREAD_ID` AS `THREAD_ID`,`tl`.`CREATE_UNIXTIME` AS `CREATE_UNIXTIME`,`tl`.`OUTSIDE_WALL_CONNECT_TYPE` AS `OUTSIDE_WALL_CONNECT_TYPE` from (((select `MAZE_FIELD`.`ID` AS `ID`,`MAZE_FIELD`.`CELL_TYPE` AS `CELL_TYPE`,`MAZE_FIELD`.`CELL_OWNER_THREAD_ID` AS `CELL_OWNER_THREAD_ID` from `MAZE_FIELD` where `MAZE_FIELD`.`CELL_OWNER_THREAD_ID` is not null) `mf` join `START_POINTS_VIEW` `spv` on(`mf`.`ID` = `spv`.`CELL_ID`)) join `THREAD_LIST` `tl` on(`tl`.`ID` = `mf`.`CELL_OWNER_THREAD_ID`)) */;
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

-- Dump completed on 2026-01-25 16:40:05
