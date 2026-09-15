import sys
import json
import os
import base64
from policy_manager import insert_policy, update_policy, delete_policy, get_policies, get_policy, get_config

def eprint(*args, **kwargs):
    print(*args, file=sys.stderr, **kwargs)

class ConfigCommand:
    def execute(self, args):
        if args.subcommand == "add":
            self._add_config(args)
        elif args.subcommand == "update":
            self._update_config(args)
        elif args.subcommand == "remove":
            self._rem_config(args)
        elif args.subcommand == "list":
            self._list_config(args)
        elif args.subcommand == "show":
            self._show_config(args)
        elif args.subcommand == "get":
            self._get_config(args)
        else:
            eprint(f"Unknown subcommand: {args.subcommand}")
            sys.exit(1)

    def _add_config(self, args):
        filepath = args.filepath

        if not os.path.exists(filepath):
            eprint("ERROR: file %s doesn't exists" % filepath)
            sys.exit(1)

        try:
            with open(filepath, 'r') as f:
                text = f.read()
        except:
            eprint("ERROR: Cannot read file %s" % filepath)
            sys.exit(1)

        try:
            json_object = json.loads(text)
        except ValueError as e:
            eprint("ERROR: file %s is not a valid json file" % filepath)
            sys.exit(1)

        result = insert_policy(args.name, text);
        if result:
            print(f"ADDED config {args.name}")
        else:
            print(f"FAILED to ADD config {args.name}")
            sys.exit(1)

    def _update_config(self, args):
        filepath = args.filepath

        if not os.path.exists(filepath):
            eprint("ERROR: file %s doesn't exists" % filepath)
            sys.exit(1)

        try:
            with open(filepath, 'r') as f:
                text = f.read()
        except:
            eprint("ERROR: Cannot read file %s" % filepath)
            sys.exit(1)

        try:
            json_object = json.loads(text)
        except ValueError as e:
            eprint("ERROR: file %s is not a valid json file" % filepath)
            sys.exit(1)

        result = update_policy(args.name, text);
        if result:
            print(f"UPDATED config {args.name}")
        else:
            print(f"FAILED to UPDATE config {args.name}")
            sys.exit(1)

    def _rem_config(self, args):
        result = delete_policy(args.name);
        if result:
            print(f"REMOVED config {args.name}")
        else:
            print(f"FAILED to REMOVE config {args.name}")
            sys.exit(1)
    
    def _list_config(self, args):
        result = get_policies()
        for row in result:
            print("%s:\n%s" %(row[0], row[1]))

    def _show_config(self, args):
        result = get_policy(args.name)
        if result is None:
            eprint("ERROR: No config named %s" % args.name)
            sys.exit(1)
        else:            
            for row in result:
                print("%s:\n%s" %(args.name, row))

    def _get_config(self, args):
        config = get_config(args.job, args.system, args.account, args.user);
        print(config)
