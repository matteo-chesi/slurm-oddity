# cli.py
import argparse
from config import ConfigCommand
from rule import RuleCommand

class TaskCLI:
    def __init__(self):
        self.commands = {
            "config": ConfigCommand(),
            "rule": RuleCommand(),
        }

    def run(self):
        parser = argparse.ArgumentParser(prog='launch-control', description="Launch Control CLI")
        subparsers = parser.add_subparsers(dest="command", required=True)
        
        # config command (with subcommands)
        config_parser = subparsers.add_parser("config", help="manage configurations")
        config_subparsers = config_parser.add_subparsers(dest="subcommand", required=True)
        
        # config list
        config_list_parser = config_subparsers.add_parser("list", help="list configurations")
        config_list_parser.set_defaults(subcommand="list", func=self.commands["config"].execute)

        # config show
        config_show_parser = config_subparsers.add_parser("show", help="show a specific configuration")
        config_show_parser.add_argument("--name",
                                       required=True, 
                                       help="Name of the configuration to show")
        config_show_parser.set_defaults(subcommand="show", func=self.commands["config"].execute)

        # config add
        config_add_parser = config_subparsers.add_parser("add", help="add a new configuration")
        config_add_parser.add_argument("--name",
                                       required=True, 
                                       help="Name of the configuration to add")
        config_add_parser.add_argument("--from", 
                                       dest="filepath",
                                       required=True, 
                                       help="File path containing the new configuration")
        config_add_parser.set_defaults(subcommand="add", func=self.commands["config"].execute)
        
        # config update
        config_update_parser = config_subparsers.add_parser("update", help="update an existing configuration")
        config_update_parser.add_argument("--name",
                                       required=True, 
                                       help="Name of the configuration to update")
        config_update_parser.add_argument("--from", 
                                       dest="filepath",
                                       required=True, 
                                       help="File path containing the updated configuration")
        config_update_parser.set_defaults(subcommand="update", func=self.commands["config"].execute)
        
        # config remove
        config_remove_parser = config_subparsers.add_parser("remove", help="remove a configuration")
        config_remove_parser.add_argument("--name",
                                          required=True, 
                                          help="Name of the configuration")
        config_remove_parser.set_defaults(subcommand="remove", func=self.commands["config"].execute)
        
        # config get
        config_get_parser = config_subparsers.add_parser("get", help="collect configuration for a job")
        config_get_parser.add_argument("--job",
                                          required=True, 
                                          help="job id")
        config_get_parser.add_argument("--system",
                                          required=True, 
                                          help="System name")
        config_get_parser.add_argument("--account",
                                          required=True, 
                                          help="Account name")
        config_get_parser.add_argument("--user",
                                          required=True, 
                                          help="User name")
        config_get_parser.set_defaults(subcommand="get", func=self.commands["config"].execute)
        
        # rule command (with subcommands)
        rule_parser = subparsers.add_parser("rule", help="manage rules")
        rule_subparsers = rule_parser.add_subparsers(dest="subcommand", required=True)
        
        # rule list
        rule_list_parser = rule_subparsers.add_parser("list", help="list rules")
        rule_list_parser.set_defaults(subcommand="list", func=self.commands["rule"].execute)

        # rule add
        rule_add_parser = rule_subparsers.add_parser("add", help="add a new rules")
        rule_add_parser.add_argument("--config",
                                       required=True, 
                                       help="Name of the configuration")
        rule_add_parser.add_argument("--system", 
                                       help="System to be associated to this rule")
        rule_add_parser.add_argument("--account", 
                                       help="Account to be associated to this rule")
        rule_add_parser.add_argument("--user", 
                                       help="User to be associated to this rule")
        rule_add_parser.set_defaults(subcommand="add", func=self.commands["rule"].execute)

        # rule remove
        rule_remove_parser = rule_subparsers.add_parser("remove", help="remove a rule")
        rule_remove_parser.add_argument("--id",
                                          required=True, 
                                          help="id of the rule to remove")
        rule_remove_parser.set_defaults(subcommand="remove", func=self.commands["rule"].execute)

        args = parser.parse_args()
        args.func(args)

if __name__ == "__main__":
    TaskCLI().run()
