from flask import Flask, jsonify, request
import policy_manager
from db import init_database

app = Flask(__name__)

@app.route('/get_config', methods=["POST"])
def get_config():
    fields = request.get_json()
    job = fields["job"]
    system = fields["system"]
    account = fields["account"]
    user = fields["user"]
    config = policy_manager.get_config(job, system, account, user)
    return jsonify(config)

if __name__ == "__main__":
    init_database()
    app.run(host='0.0.0.0', port=7667, debug=False)

