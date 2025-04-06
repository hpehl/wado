# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_waco_global_optspecs
	string join \n h/help V/version
end

function __fish_waco_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_waco_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_waco_using_subcommand
	set -l cmd (__fish_waco_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c waco -n "__fish_waco_needs_command" -s h -l help -d 'Print help'
complete -c waco -n "__fish_waco_needs_command" -s V -l version -d 'Print version'
complete -c waco -n "__fish_waco_needs_command" -f -a "build" -d 'Build images'
complete -c waco -n "__fish_waco_needs_command" -f -a "push" -d 'Push images'
complete -c waco -n "__fish_waco_needs_command" -f -a "start" -d 'Start a standalone server'
complete -c waco -n "__fish_waco_needs_command" -f -a "stop" -d 'Stop a standalone server'
complete -c waco -n "__fish_waco_needs_command" -f -a "dc" -d 'Start and stop a domain controller'
complete -c waco -n "__fish_waco_needs_command" -f -a "hc" -d 'Start and stop a host controller'
complete -c waco -n "__fish_waco_needs_command" -f -a "topology" -d 'Start and stop a topology defined in YAML'
complete -c waco -n "__fish_waco_needs_command" -f -a "ps" -d 'List running standalone, domain and host controller containers'
complete -c waco -n "__fish_waco_needs_command" -f -a "console" -d 'Open the management console'
complete -c waco -n "__fish_waco_needs_command" -f -a "cli" -d 'Connect to the CLI'
complete -c waco -n "__fish_waco_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c waco -n "__fish_waco_using_subcommand build" -s u -l username -d 'The username of the management user' -r
complete -c waco -n "__fish_waco_using_subcommand build" -s p -l password -d 'The password of the management user' -r
complete -c waco -n "__fish_waco_using_subcommand build" -l chunks -d 'Build the images in chunks of this size. If not specified, the images are built in one go.' -r
complete -c waco -n "__fish_waco_using_subcommand build" -l standalone -d 'Build standalone images only'
complete -c waco -n "__fish_waco_using_subcommand build" -l domain -d 'Build domain controller and host controller images only'
complete -c waco -n "__fish_waco_using_subcommand build" -s h -l help -d 'Print help'
complete -c waco -n "__fish_waco_using_subcommand build" -s V -l version -d 'Print version'
complete -c waco -n "__fish_waco_using_subcommand push" -l chunks -d 'Push the images in chunks of this size. If not specified, the images are pushed in one go.' -r
complete -c waco -n "__fish_waco_using_subcommand push" -l standalone -d 'Push standalone images only'
complete -c waco -n "__fish_waco_using_subcommand push" -l domain -d 'Push domain controller and host controller images only'
complete -c waco -n "__fish_waco_using_subcommand push" -s h -l help -d 'Print help'
complete -c waco -n "__fish_waco_using_subcommand push" -s V -l version -d 'Print version'
complete -c waco -n "__fish_waco_using_subcommand start" -s n -l name -d 'The name of the standalone server [default: waco-sa-<major><minor>]. Not allowed when multiple versions are specified.' -r
complete -c waco -n "__fish_waco_using_subcommand start" -s p -l http -d 'The published HTTP port [default: 8<major><minor>]. Not allowed when multiple versions are specified.' -r
complete -c waco -n "__fish_waco_using_subcommand start" -s m -l management -d 'The published management port [default: 9<major><minor>]. Not allowed when multiple versions are specified.' -r
complete -c waco -n "__fish_waco_using_subcommand start" -s o -l offset -d 'The offset added to the published HTTP and management ports. Not allowed when multiple versions are specified.' -r
complete -c waco -n "__fish_waco_using_subcommand start" -l operations -d 'A comma seperated list of operations to bootstrap the standalone server. Can be provided multiple times.' -r
complete -c waco -n "__fish_waco_using_subcommand start" -l cli -d 'A file with operations to bootstrap the standalone server' -r
complete -c waco -n "__fish_waco_using_subcommand start" -s h -l help -d 'Print help'
complete -c waco -n "__fish_waco_using_subcommand start" -s V -l version -d 'Print version'
complete -c waco -n "__fish_waco_using_subcommand stop" -s n -l name -d 'The name of the standalone server [default: waco-sa-<major><minor>]' -r
complete -c waco -n "__fish_waco_using_subcommand stop" -s a -l all -d 'Stop all running standalone servers. If specified with a version, stop all running standalone servers of that version.'
complete -c waco -n "__fish_waco_using_subcommand stop" -s h -l help -d 'Print help'
complete -c waco -n "__fish_waco_using_subcommand stop" -s V -l version -d 'Print version'
complete -c waco -n "__fish_waco_using_subcommand dc; and not __fish_seen_subcommand_from start stop help" -s h -l help -d 'Print help'
complete -c waco -n "__fish_waco_using_subcommand dc; and not __fish_seen_subcommand_from start stop help" -s V -l version -d 'Print version'
complete -c waco -n "__fish_waco_using_subcommand dc; and not __fish_seen_subcommand_from start stop help" -f -a "start" -d 'Start a domain controller'
complete -c waco -n "__fish_waco_using_subcommand dc; and not __fish_seen_subcommand_from start stop help" -f -a "stop" -d 'Stop a domain controller'
complete -c waco -n "__fish_waco_using_subcommand dc; and not __fish_seen_subcommand_from start stop help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c waco -n "__fish_waco_using_subcommand dc; and __fish_seen_subcommand_from start" -s n -l name -d 'The name of the domain controller [default: waco-dc-<major><minor>]. Not allowed when multiple versions are specified.' -r
complete -c waco -n "__fish_waco_using_subcommand dc; and __fish_seen_subcommand_from start" -s p -l http -d 'The published HTTP port [default: 8<major><minor>]. Not allowed when multiple versions are specified.' -r
complete -c waco -n "__fish_waco_using_subcommand dc; and __fish_seen_subcommand_from start" -s m -l management -d 'The published management port [default: 9<major><minor>]. Not allowed when multiple versions are specified.' -r
complete -c waco -n "__fish_waco_using_subcommand dc; and __fish_seen_subcommand_from start" -s o -l offset -d 'The offset added to the published HTTP and management ports. Not allowed when multiple versions are specified.' -r
complete -c waco -n "__fish_waco_using_subcommand dc; and __fish_seen_subcommand_from start" -s s -l server -d 'Manage servers of the domain controller. Servers are specified as a comma seperated list of <name>[:<server-group>][:<offset>][:start].  The option can be specified multiple times.  <name>          The name of the server. This part is mandatory and must be first.                 All other parts are optional. <server-group>  The name of the server group. Allowed values are \'main-server-group\' or \'msg\',                 and \'other-server-group\' or \'osg\'. If not specified, \'main-server-group\' is used. <offset>        The port offset. If not specified, 100 is used from the second server onwards. start           Whether to start the server.' -r
complete -c waco -n "__fish_waco_using_subcommand dc; and __fish_seen_subcommand_from start" -l operations -d 'A comma seperated list of operations to bootstrap the domain controller. Can be provided multiple times.' -r
complete -c waco -n "__fish_waco_using_subcommand dc; and __fish_seen_subcommand_from start" -l cli -d 'A file with operations to bootstrap the domain controller' -r
complete -c waco -n "__fish_waco_using_subcommand dc; and __fish_seen_subcommand_from start" -s h -l help -d 'Print help'
complete -c waco -n "__fish_waco_using_subcommand dc; and __fish_seen_subcommand_from start" -s V -l version -d 'Print version'
complete -c waco -n "__fish_waco_using_subcommand dc; and __fish_seen_subcommand_from stop" -s n -l name -d 'The name of the domain controller [default: waco-dc-<major><minor>]' -r
complete -c waco -n "__fish_waco_using_subcommand dc; and __fish_seen_subcommand_from stop" -s a -l all -d 'Stop all running domain controllers. If specified with a version, stop all running domain controllers of that version.'
complete -c waco -n "__fish_waco_using_subcommand dc; and __fish_seen_subcommand_from stop" -s h -l help -d 'Print help'
complete -c waco -n "__fish_waco_using_subcommand dc; and __fish_seen_subcommand_from stop" -s V -l version -d 'Print version'
complete -c waco -n "__fish_waco_using_subcommand dc; and __fish_seen_subcommand_from help" -f -a "start" -d 'Start a domain controller'
complete -c waco -n "__fish_waco_using_subcommand dc; and __fish_seen_subcommand_from help" -f -a "stop" -d 'Stop a domain controller'
complete -c waco -n "__fish_waco_using_subcommand dc; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c waco -n "__fish_waco_using_subcommand hc; and not __fish_seen_subcommand_from start stop help" -s h -l help -d 'Print help'
complete -c waco -n "__fish_waco_using_subcommand hc; and not __fish_seen_subcommand_from start stop help" -s V -l version -d 'Print version'
complete -c waco -n "__fish_waco_using_subcommand hc; and not __fish_seen_subcommand_from start stop help" -f -a "start" -d 'Start a host controller'
complete -c waco -n "__fish_waco_using_subcommand hc; and not __fish_seen_subcommand_from start stop help" -f -a "stop" -d 'Stop a host controller'
complete -c waco -n "__fish_waco_using_subcommand hc; and not __fish_seen_subcommand_from start stop help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c waco -n "__fish_waco_using_subcommand hc; and __fish_seen_subcommand_from start" -s n -l name -d 'The name of the host controller [default: waco-hc-<major><minor>]. Not allowed when multiple versions are specified.' -r
complete -c waco -n "__fish_waco_using_subcommand hc; and __fish_seen_subcommand_from start" -s d -l domain-controller -d 'The name of the domain controller [default: waco-dc-<major><minor>]. Required if different versions are specified.' -r
complete -c waco -n "__fish_waco_using_subcommand hc; and __fish_seen_subcommand_from start" -s u -l username -d 'The username to connect to the domain controller' -r
complete -c waco -n "__fish_waco_using_subcommand hc; and __fish_seen_subcommand_from start" -s p -l password -d 'The password to connect to the domain controller' -r
complete -c waco -n "__fish_waco_using_subcommand hc; and __fish_seen_subcommand_from start" -s s -l server -d 'Manage servers of the host controller. Servers are specified as a comma seperated list of <name>[:<server-group>][:<offset>][:start].  The option can be specified multiple times.  <name>          The name of the server. This part is mandatory and must be first.                 All other parts are optional. <server-group>  The name of the server group. Allowed values are \'main-server-group\' or \'msg\',                 and \'other-server-group\' or \'osg\'. If not specified, \'main-server-group\' is used. <offset>        The port offset. If not specified, 100 is used from the second server onwards. start           Whether to start the server.' -r
complete -c waco -n "__fish_waco_using_subcommand hc; and __fish_seen_subcommand_from start" -l operations -d 'A comma seperated list of operations to bootstrap the host controller. Can be provided multiple times.' -r
complete -c waco -n "__fish_waco_using_subcommand hc; and __fish_seen_subcommand_from start" -l cli -d 'A file with operations to bootstrap the host controller' -r
complete -c waco -n "__fish_waco_using_subcommand hc; and __fish_seen_subcommand_from start" -s h -l help -d 'Print help'
complete -c waco -n "__fish_waco_using_subcommand hc; and __fish_seen_subcommand_from start" -s V -l version -d 'Print version'
complete -c waco -n "__fish_waco_using_subcommand hc; and __fish_seen_subcommand_from stop" -s n -l name -d 'The name of the host controller [default: waco-hc-<major><minor>]' -r
complete -c waco -n "__fish_waco_using_subcommand hc; and __fish_seen_subcommand_from stop" -s a -l all -d 'Stop all running host controllers. If specified with a version, stop all running host controllers of that version.'
complete -c waco -n "__fish_waco_using_subcommand hc; and __fish_seen_subcommand_from stop" -s h -l help -d 'Print help'
complete -c waco -n "__fish_waco_using_subcommand hc; and __fish_seen_subcommand_from stop" -s V -l version -d 'Print version'
complete -c waco -n "__fish_waco_using_subcommand hc; and __fish_seen_subcommand_from help" -f -a "start" -d 'Start a host controller'
complete -c waco -n "__fish_waco_using_subcommand hc; and __fish_seen_subcommand_from help" -f -a "stop" -d 'Stop a host controller'
complete -c waco -n "__fish_waco_using_subcommand hc; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c waco -n "__fish_waco_using_subcommand topology; and not __fish_seen_subcommand_from start stop help" -s h -l help -d 'Print help'
complete -c waco -n "__fish_waco_using_subcommand topology; and not __fish_seen_subcommand_from start stop help" -s V -l version -d 'Print version'
complete -c waco -n "__fish_waco_using_subcommand topology; and not __fish_seen_subcommand_from start stop help" -f -a "start" -d 'Start a topology'
complete -c waco -n "__fish_waco_using_subcommand topology; and not __fish_seen_subcommand_from start stop help" -f -a "stop" -d 'Stop a topology'
complete -c waco -n "__fish_waco_using_subcommand topology; and not __fish_seen_subcommand_from start stop help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c waco -n "__fish_waco_using_subcommand topology; and __fish_seen_subcommand_from start" -s h -l help -d 'Print help'
complete -c waco -n "__fish_waco_using_subcommand topology; and __fish_seen_subcommand_from start" -s V -l version -d 'Print version'
complete -c waco -n "__fish_waco_using_subcommand topology; and __fish_seen_subcommand_from stop" -s h -l help -d 'Print help'
complete -c waco -n "__fish_waco_using_subcommand topology; and __fish_seen_subcommand_from stop" -s V -l version -d 'Print version'
complete -c waco -n "__fish_waco_using_subcommand topology; and __fish_seen_subcommand_from help" -f -a "start" -d 'Start a topology'
complete -c waco -n "__fish_waco_using_subcommand topology; and __fish_seen_subcommand_from help" -f -a "stop" -d 'Stop a topology'
complete -c waco -n "__fish_waco_using_subcommand topology; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c waco -n "__fish_waco_using_subcommand ps" -s h -l help -d 'Print help'
complete -c waco -n "__fish_waco_using_subcommand ps" -s V -l version -d 'Print version'
complete -c waco -n "__fish_waco_using_subcommand console" -s n -l name -d 'The name of the standalone server or domain controller [default: waco-sa|dc-<major><minor>]. Not allowed when multiple versions are specified.' -r
complete -c waco -n "__fish_waco_using_subcommand console" -s m -l management -d 'The published management port. Not allowed when multiple versions are specified.' -r
complete -c waco -n "__fish_waco_using_subcommand console" -s h -l help -d 'Print help'
complete -c waco -n "__fish_waco_using_subcommand console" -s V -l version -d 'Print version'
complete -c waco -n "__fish_waco_using_subcommand cli" -s n -l name -d 'The name of the standalone server or domain controller [default: waco-sa|dc-<major><minor>]. Not allowed when multiple versions are specified.' -r
complete -c waco -n "__fish_waco_using_subcommand cli" -s m -l management -d 'The published management port' -r
complete -c waco -n "__fish_waco_using_subcommand cli" -s u -l username -d 'The username to connect to the CLI' -r
complete -c waco -n "__fish_waco_using_subcommand cli" -s p -l password -d 'The password to connect to the CLI' -r
complete -c waco -n "__fish_waco_using_subcommand cli" -s h -l help -d 'Print help'
complete -c waco -n "__fish_waco_using_subcommand cli" -s V -l version -d 'Print version'
complete -c waco -n "__fish_waco_using_subcommand help; and not __fish_seen_subcommand_from build push start stop dc hc topology ps console cli help" -f -a "build" -d 'Build images'
complete -c waco -n "__fish_waco_using_subcommand help; and not __fish_seen_subcommand_from build push start stop dc hc topology ps console cli help" -f -a "push" -d 'Push images'
complete -c waco -n "__fish_waco_using_subcommand help; and not __fish_seen_subcommand_from build push start stop dc hc topology ps console cli help" -f -a "start" -d 'Start a standalone server'
complete -c waco -n "__fish_waco_using_subcommand help; and not __fish_seen_subcommand_from build push start stop dc hc topology ps console cli help" -f -a "stop" -d 'Stop a standalone server'
complete -c waco -n "__fish_waco_using_subcommand help; and not __fish_seen_subcommand_from build push start stop dc hc topology ps console cli help" -f -a "dc" -d 'Start and stop a domain controller'
complete -c waco -n "__fish_waco_using_subcommand help; and not __fish_seen_subcommand_from build push start stop dc hc topology ps console cli help" -f -a "hc" -d 'Start and stop a host controller'
complete -c waco -n "__fish_waco_using_subcommand help; and not __fish_seen_subcommand_from build push start stop dc hc topology ps console cli help" -f -a "topology" -d 'Start and stop a topology defined in YAML'
complete -c waco -n "__fish_waco_using_subcommand help; and not __fish_seen_subcommand_from build push start stop dc hc topology ps console cli help" -f -a "ps" -d 'List running standalone, domain and host controller containers'
complete -c waco -n "__fish_waco_using_subcommand help; and not __fish_seen_subcommand_from build push start stop dc hc topology ps console cli help" -f -a "console" -d 'Open the management console'
complete -c waco -n "__fish_waco_using_subcommand help; and not __fish_seen_subcommand_from build push start stop dc hc topology ps console cli help" -f -a "cli" -d 'Connect to the CLI'
complete -c waco -n "__fish_waco_using_subcommand help; and not __fish_seen_subcommand_from build push start stop dc hc topology ps console cli help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c waco -n "__fish_waco_using_subcommand help; and __fish_seen_subcommand_from dc" -f -a "start" -d 'Start a domain controller'
complete -c waco -n "__fish_waco_using_subcommand help; and __fish_seen_subcommand_from dc" -f -a "stop" -d 'Stop a domain controller'
complete -c waco -n "__fish_waco_using_subcommand help; and __fish_seen_subcommand_from hc" -f -a "start" -d 'Start a host controller'
complete -c waco -n "__fish_waco_using_subcommand help; and __fish_seen_subcommand_from hc" -f -a "stop" -d 'Stop a host controller'
complete -c waco -n "__fish_waco_using_subcommand help; and __fish_seen_subcommand_from topology" -f -a "start" -d 'Start a topology'
complete -c waco -n "__fish_waco_using_subcommand help; and __fish_seen_subcommand_from topology" -f -a "stop" -d 'Stop a topology'
