from db import get_db
import json

def insert_policy(name, data):
    db = get_db()
    cursor = db.cursor()

    statement = "INSERT INTO policies(name, data) VALUES (?, ?)"
    try:
        cursor.execute(statement, [name, data])
        db.commit()
    except:
        return False

    return True

def update_policy(name, data):
    db = get_db()
    cursor = db.cursor()
    statement = "UPDATE policies SET data = ? WHERE name = ?"
    cursor.execute(statement, [data, name])
    db.commit()
    return True

def delete_policy(name):
    db = get_db()

    policy = get_policy(name)
    if policy is None:
        return False

    cursor = db.cursor()
    statement = "DELETE FROM policies WHERE name = ?"
    try:
        cursor.execute(statement, [name])
        db.commit()
    except:
        return False

    return True

def get_policy(name):
    db = get_db()
    cursor = db.cursor()
    statement = "SELECT data FROM policies WHERE name = ?"
    cursor.execute(statement, [name])
    return cursor.fetchone()

def get_policies():
    db = get_db()
    cursor = db.cursor()
    query = "SELECT name, data FROM policies"
    cursor.execute(query)
    return cursor.fetchall()

def insert_rule(policy, system, account, user):
    db = get_db()
    cursor = db.cursor()
    statement = "INSERT INTO rules(policy, system, account, user) VALUES (?, ?, ?, ?)"
    try:
        cursor.execute(statement, [policy, system, account, user])
        db.commit()
    except:
        return False
    return True

def update_rule(id, policy, system, account, user):
    db = get_db()
    cursor = db.cursor()
    statement = "UPDATE rules SET policy = ?, system = ?, account = ?, user = ? WHERE id = ?"
    cursor.execute(statement, [plugin, engine, name])
    db.commit()
    return True

def delete_rule(id):
    db = get_db()
    cursor = db.cursor()
    statement = "DELETE FROM rules WHERE id = ?"
    cursor.execute(statement, [id])
    db.commit()
    return True

def get_rule(id):
    db = get_db()
    cursor = db.cursor()
    statement = "SELECT id, policy, system, account, user FROM rules WHERE id = ?"
    cursor.execute(statement, [name])
    return cursor.fetchone()

def get_rules():
    db = get_db()
    cursor = db.cursor()
    query = "SELECT id, policy, system, account, user FROM rules"
    cursor.execute(query)
    return cursor.fetchall()

def get_config_name(job, system, account, user):
    db = get_db()
    cursor = db.cursor()
    query = "SELECT policy FROM rules WHERE system = ? AND account = ? AND user = ?"
    cursor.execute(query, [system, account, user])
    result = cursor.fetchall()
    if result:
        return result[0][0]

    cursor.execute(query, [system, "ALL", user])
    result = cursor.fetchall()
    if result:
        return result[0][0]

    cursor.execute(query, [system, account, "ALL"])
    result = cursor.fetchall()
    if result:
        return result[0][0]

    cursor.execute(query, [system, "ALL", "ALL"])
    result = cursor.fetchall()
    if result:
        return result[0][0]

    cursor.execute(query, ["ALL", account, user])
    result = cursor.fetchall()
    if result:
        return result[0][0]
    
    cursor.execute(query, ["ALL", "ALL", user])
    result = cursor.fetchall()
    if result:
        return result[0][0]

    cursor.execute(query, ["ALL", account, "ALL"])
    result = cursor.fetchall()
    if result:
        return result[0][0]

    cursor.execute(query, ["ALL", "ALL", "ALL"])
    result = cursor.fetchall()
    if result:
        return result[0][0]
    else:
        return None

def get_config(job, system, account, user):
    config = {}
    config_name = get_config_name(job, system, account, user)
    if config_name:
        policy = get_policy(config_name);
        config = json.loads(policy[0])
    return config 

