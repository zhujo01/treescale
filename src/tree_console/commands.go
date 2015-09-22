package tree_console
import (
	"github.com/spf13/cobra"
	"fmt"
)

var (
	TreeScaleCMD	*cobra.Command
)

func init() {
	TreeScaleCMD = &cobra.Command{
		Use: 	"treescale [commands]",
		Short:	"Scaling and Resource Managaement evented system, based on Mathematical Tree and Graph",
		Long:	"",
	}

	// Adding Flags


	// List of commands to execute
	version := &cobra.Command{
		Use: "version",
		Aliases: []string{"v"},
		Short: "Prints version of program",
		Run: func(cmd *cobra.Command, args []string) {
			fmt.Println("TreeScale Version 0.001")
			fmt.Println("Copyright TreeScale Inc.")
		},
	}

	// Configuration commands
	config := &cobra.Command{
		Use: 	"config [commands]",
		Short:	"Command for handling configuration tools",
	}
		compile_config := &cobra.Command{
			Use: "compile [options]",
			Short: "Compiles multiple config files",
			Long: "Compiles multiple config files into one single TreeScale database for sending to nodes",
			Run: CompileConfig,
		}
		compile_config.Flags().StringP("type", "t", "toml", "Configuration file format [toml, json, yaml] default is TOML")
		compile_config.Flags().StringSliceP("path", "p", []string{"."}, "Give a Path to directory of configuration files")
		compile_config.Flags().StringSliceP("files", "f", []string{"console.toml", "treescale.toml"}, "Give file path list of configuration files")
		compile_config.Flags().StringP("out", "o", "tree.db", "Output file for compiled config files")

		restore_db := &cobra.Command{
			Use: "restore [options]",
			Short: "Restore compiled config to local database",
			Long: "This command restoring compiled and dumped file to local database for running TreeScale as a daemon",
			Run: RestoreFromConfigDump,
		}
		restore_db.Flags().StringP("file", "f", "tree.db", "Path to dumped file")

	config.AddCommand(compile_config, restore_db)


	// Node console commands
	node_cmd := &cobra.Command{
		Use: "node [options]",
		Short: "Command for running node as a Tree component",
		Long: `This command starting Tree Networking and Event handling
		Note: before running this command make sure this node have Tree Dabase restored, or transfered
		`,
		Run: HandleNodeCommand,
	}
	node_cmd.Flags().BoolP("daemon", "d", false, "Run Node in daemon mode")
	node_cmd.Flags().StringP("name", "n", "", "Set Node name for running it (needs to set for the first time and if it needs to be changed)")
	node_cmd.Flags().String("set-name", "", "Set Node name as a current node name in database")


	// API commands
	api_cmd := &cobra.Command{
		Use: "api [commands]",
		Short: "Send API commands to nodes and get results",
	}
		api_cmd_exec := &cobra.Command{
			Use: "exec [options]",
			Short: "Execute shell commands on specific Nodes",
			Run: HandleApiExec,
		}
		api_cmd_exec.Flags().StringSliceP("nodes", "n", []string{""}, "Array of Nodes for sending API command")
		api_cmd_exec.Flags().StringP("cmd", "c", "uname", "Shell command to execute")
	api_cmd.AddCommand(api_cmd_exec)


	TreeScaleCMD.AddCommand(version, config, node_cmd, api_cmd)
}