import sys
import json
from policy_manager import insert_rule, delete_rule, get_rules, get_policies

class RuleCommand:
    def execute(self, args):
        if args.subcommand == "add":
            self._add(args)
        elif args.subcommand == "remove":
            self._rem(args)
        elif args.subcommand == "list":
            self._list(args)
        else:
            print(f"Unknown subcommand: {args.subcommand}")
            sys.exit(1)

    def _add(self, args):
        configs = get_policies()
        config_names = [];
        for c in configs:
            config_names.append(c[0]);
        if args.config not in config_names:
            print(f"ERROR: {args.config} doesn't exist")
            sys.exit(1)

        if args.system == None:
            args.system = "ALL"
        if args.account == None:
            args.account = "ALL"
        if args.user == None:
            args.user = "ALL"
            
        result = insert_rule(args.config, args.system, args.account, args.user);
        if result:
            print(f"ADDED rule")
        else:
            print(f"FAILED to ADD rule")
            sys.exit(1)

    def _rem(self, args):
        result = delete_rule(args.id);
        if result:
            print(f"REMOVED rule {args.id}")
        else:
            print(f"FAILED to REMOVE rule {args.id}")
            sys.exit(1)
    
    def _list(self, args):
        result = get_rules()
        print("%-15s %-15s %-15s %-15s %-15s\n" % ("ID", "CONFIG", "SYSTEM", "ACCOUNT", "USER"))
        for row in result:
            system = row[2] or "-";
            account = row[3] or "-";
            user = row[4] or "-";
            print("%-15s %-15s %-15s %-15s %-15s" %(row[0], row[1], system, account, user))

