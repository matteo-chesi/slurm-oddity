PRAGMA foreign_keys=OFF;
BEGIN TRANSACTION;
CREATE TABLE policies (
                name TEXT PRIMARY KEY,
                data BLOB NOT NULL
            );
INSERT INTO policies VALUES('green',replace('{\n  "stage1_repository": "/repo",\n  "stage1_version": "v0.0.1-green"\n}\n','\n',char(10)));
INSERT INTO policies VALUES('blue',replace('{\n  "stage1_repository": "/repo",\n  "stage1_version": "v0.0.1-blue"\n}\n','\n',char(10)));
INSERT INTO policies VALUES('yellow',replace('{\n  "stage1_repository": "/repo",\n  "stage1_version": "v0.0.1-yellow"\n}\n','\n',char(10)));
CREATE TABLE rules (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                policy TEXT NOT_NULL,
                system TEXT,
                account TEXT,
                user TEXT,
                unique (system,account,user)
            );
INSERT INTO rules VALUES(1,'blue','zinal','ALL','ALL');
INSERT INTO rules VALUES(2,'yellow','ALL','ALL','vagrant');
DELETE FROM sqlite_sequence;
INSERT INTO sqlite_sequence VALUES('rules',2);
COMMIT;
