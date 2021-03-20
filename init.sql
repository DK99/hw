USE hedgewars;
CREATE TABLE `users_roles` (
                               `uid` int(10) unsigned NOT NULL DEFAULT '0',
                               `rid` int(10) unsigned NOT NULL DEFAULT '0',
                               PRIMARY KEY (`uid`,`rid`)
) ENGINE=MyISAM DEFAULT CHARSET=utf8;
CREATE TABLE `users` (
                         `uid` int(10) unsigned NOT NULL DEFAULT '0',
                         `name` varchar(60) DEFAULT NULL,
                         `pass` varchar(32) NOT NULL DEFAULT '',
                         `mail` varchar(64) DEFAULT '',
                         `mode` tinyint(4) NOT NULL DEFAULT '0',
                         `sort` tinyint(4) DEFAULT '0',
                         `threshold` tinyint(4) DEFAULT '0',
                         `theme` varchar(255) NOT NULL DEFAULT '',
                         `signature` varchar(255) NOT NULL DEFAULT '',
                         `created` int(11) NOT NULL DEFAULT '0',
                         `access` int(11) NOT NULL DEFAULT '0',
                         `login` int(11) NOT NULL DEFAULT '0',
                         `status` tinyint(4) NOT NULL DEFAULT '0',
                         `timezone` varchar(8) DEFAULT NULL,
                         `language` varchar(12) NOT NULL DEFAULT '',
                         `picture` varchar(255) NOT NULL DEFAULT '',
                         `init` varchar(64) DEFAULT '',
                         `data` longtext,
                         PRIMARY KEY (`uid`),
                         UNIQUE KEY `name` (`name`),
                         KEY `created` (`created`),
                         KEY `access` (`access`),
                         KEY `uid` (`uid`)
) ENGINE=MyISAM DEFAULT CHARSET=utf8;
CREATE TABLE `gameserver_stats` (
                                    `players` smallint(5) unsigned NOT NULL DEFAULT '0',
                                    `rooms` smallint(5) unsigned NOT NULL DEFAULT '0',
                                    `last_update` int(11) NOT NULL DEFAULT '0'
) ENGINE=MyISAM DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci;
CREATE TABLE `achievements` (
                                `id` int(11) NOT NULL AUTO_INCREMENT,
                                `time` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
                                `typeid` int(11) NOT NULL,
                                `userid` int(11) NOT NULL,
                                `value` int(11) DEFAULT NULL,
                                `filename` varchar(64) DEFAULT NULL,
                                `location` varchar(64) DEFAULT NULL,
                                `protocol` int(11) DEFAULT NULL,
                                PRIMARY KEY (`id`),
                                UNIQUE KEY `achievements_unique` (`time`,`typeid`,`userid`,`value`,`location`),
                                KEY `typeid` (`typeid`),
                                KEY `userid` (`userid`)
) ENGINE=MyISAM AUTO_INCREMENT=49213 DEFAULT CHARSET=utf8;
CREATE TABLE `rating_games` (
                                `id` int(11) NOT NULL AUTO_INCREMENT,
                                `script` varchar(64) DEFAULT NULL,
                                `protocol` int(11) DEFAULT NULL,
                                `filename` varchar(64) DEFAULT NULL,
                                `time` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
                                `vamp` tinyint(1) DEFAULT NULL,
                                `ropes` tinyint(1) DEFAULT NULL,
                                `infattacks` tinyint(1) DEFAULT NULL,
                                PRIMARY KEY (`id`),
                                UNIQUE KEY `filename` (`filename`)
) ENGINE=MyISAM AUTO_INCREMENT=192541 DEFAULT CHARSET=utf8;
CREATE TABLE `rating_players` (
                                  `userid` int(11) DEFAULT NULL,
                                  `gameid` int(11) NOT NULL,
                                  `place` int(11) DEFAULT NULL,
                                  KEY `userid` (`userid`),
                                  KEY `gameid` (`gameid`)
) ENGINE=MyISAM DEFAULT CHARSET=utf8;