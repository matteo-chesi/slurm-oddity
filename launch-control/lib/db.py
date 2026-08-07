import sqlite3
import pathlib
path = pathlib.Path(__file__).parent.resolve()

DATABASE_NAME = str(path.parent) + "/db/launch-control.db"

def get_db():
    conn = sqlite3.connect(DATABASE_NAME)
    return conn

def init_database():
    tables = [
        """CREATE TABLE IF NOT EXISTS policies (
                name TEXT PRIMARY KEY,
                data TEXT NOT NULL
            );
            """,
        """CREATE TABLE IF NOT EXISTS rules (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                policy TEXT NOT_NULL,
                system TEXT,
                account TEXT,
                user TEXT,
                unique (system,account,user)
            );
            """
    ]
    db = get_db()
    cursor = db.cursor()
    for table in tables:
        cursor.execute(table)
