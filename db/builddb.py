import sqlite3
import argparse
from pathlib import Path

def create_db(db_path, schema_path):
    conn = sqlite3.connect(db_path)

    try:
        with open(schema_path, "r") as sqlfile:
            script = sqlfile.read()

            conn.executescript(script)
            print("Database created!")
            conn.commit()

    except sqlite3.Error as e:
        print("An error ocurred while creating the db: " + e)
        conn.rollback()

    finally:
        conn.close()

def drop_db(db_path):
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()

    try:
        cursor.execute('''
            PRAGMA foreign_keys = OFF;
        ''')

        cursor.execute("""
            SELECT 'DROP ' || type || ' IF EXISTS "' || name || '";' 
            FROM sqlite_schema 
            WHERE type IN ('table', 'view') 
              AND name NOT LIKE 'sqlite_%';
        """)
        
        drop_statements = [row[0] for row in cursor.fetchall()]
        
        for statement in drop_statements:
            cursor.execute(statement)
            
        cursor.execute("PRAGMA foreign_keys = ON;")
        cursor.execute("VACUUM;")
        
        conn.commit()
        print("Database dropped successfully!")
    except sqlite.Error as e:
        conn.rollback()
        print("An error occurred while wiping the db: " + e)
    finally:
        conn.close()

def seed_db(db_path, seed_path):
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()

    try:
        with open(seed_path, "r") as sqlfile:
            script = sqlfile.read()

            conn.executescript(script)
            print("Database seeded!")

    except sqlite3.Error as e:
        print(t"An error ocurred while seeding the db: " + e)

    finally:
        conn.close()

def main(command, dbpath, schema, seed, append):
    match command:
        case "create":
            return create_db(dbpath, schema)
        case "drop":
            return drop_db(dbpath)
        case "seed":
            if (append):
                return seed_db(dbpath, seed)
            else:
                drop_db(dbpath)
                create_db(dbpath, schema)
                return seed_db(dbpath, seed)


if __name__ == "__main__":

    SCRIPT_DIR = Path(__file__).resolve().parent

    DB_PATH = SCRIPT_DIR / "cardoor.sqlite3"
    SCHEMA_PATH = SCRIPT_DIR / "CARDatabase.sql"
    SEED_PATH = SCRIPT_DIR / "seed.sql"
    parser = argparse.ArgumentParser()

    parser.add_argument("command", type=str, help="Command to be executed on the db: create, drop, seed")
    parser.add_argument("-d", "--dbpath", type=Path, default=DB_PATH, help="Path to SQLite DB")
    parser.add_argument("-c", "--schema", type=Path, default=SCHEMA_PATH, help="Path to SQL schema")
    parser.add_argument("-s", "--seed", type=Path, default=SEED_PATH, help="Path to the seed data")
    parser.add_argument("-a", "--append", action="store_true", help="Append to the database instead of doing a fresh seed")


    args = parser.parse_args()
    print(args.dbpath)

    main(args.command, args.dbpath, args.schema, args.seed, args.append)