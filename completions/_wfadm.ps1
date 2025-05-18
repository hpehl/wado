
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'wfadm' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'wfadm'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'wfadm' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('build', 'build', [CompletionResultType]::ParameterValue, 'Build images')
            [CompletionResult]::new('push', 'push', [CompletionResultType]::ParameterValue, 'Push images')
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start a standalone server')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop a standalone server')
            [CompletionResult]::new('dc', 'dc', [CompletionResultType]::ParameterValue, 'Start and stop a domain controller')
            [CompletionResult]::new('hc', 'hc', [CompletionResultType]::ParameterValue, 'Start and stop a host controller')
            [CompletionResult]::new('topology', 'topology', [CompletionResultType]::ParameterValue, 'Start and stop a topology defined in YAML')
            [CompletionResult]::new('images', 'images', [CompletionResultType]::ParameterValue, 'List all available standalone, domain and host controller images')
            [CompletionResult]::new('ps', 'ps', [CompletionResultType]::ParameterValue, 'List running standalone, domain and host controller containers')
            [CompletionResult]::new('console', 'console', [CompletionResultType]::ParameterValue, 'Open the management console')
            [CompletionResult]::new('cli', 'cli', [CompletionResultType]::ParameterValue, 'Connect to the CLI')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'wfadm;build' {
            [CompletionResult]::new('-u', '-u', [CompletionResultType]::ParameterName, 'The username of the management user')
            [CompletionResult]::new('--username', '--username', [CompletionResultType]::ParameterName, 'The username of the management user')
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'The password of the management user')
            [CompletionResult]::new('--password', '--password', [CompletionResultType]::ParameterName, 'The password of the management user')
            [CompletionResult]::new('--chunks', '--chunks', [CompletionResultType]::ParameterName, 'Build the images in chunks of this size. If not specified, the images are built in one go.')
            [CompletionResult]::new('--standalone', '--standalone', [CompletionResultType]::ParameterName, 'Build standalone images only')
            [CompletionResult]::new('--domain', '--domain', [CompletionResultType]::ParameterName, 'Build domain controller and host controller images only')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'wfadm;push' {
            [CompletionResult]::new('--chunks', '--chunks', [CompletionResultType]::ParameterName, 'Push the images in chunks of this size. If not specified, the images are pushed in one go.')
            [CompletionResult]::new('--standalone', '--standalone', [CompletionResultType]::ParameterName, 'Push standalone images only')
            [CompletionResult]::new('--domain', '--domain', [CompletionResultType]::ParameterName, 'Push domain controller and host controller images only')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'wfadm;start' {
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'The name of the standalone server [default: wfadm-sa-<major><minor>]. Not allowed when multiple versions are specified.')
            [CompletionResult]::new('--name', '--name', [CompletionResultType]::ParameterName, 'The name of the standalone server [default: wfadm-sa-<major><minor>]. Not allowed when multiple versions are specified.')
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'The published HTTP port [default: 8<major><minor>]. Not allowed when multiple versions are specified.')
            [CompletionResult]::new('--http', '--http', [CompletionResultType]::ParameterName, 'The published HTTP port [default: 8<major><minor>]. Not allowed when multiple versions are specified.')
            [CompletionResult]::new('-m', '-m', [CompletionResultType]::ParameterName, 'The published management port [default: 9<major><minor>]. Not allowed when multiple versions are specified.')
            [CompletionResult]::new('--management', '--management', [CompletionResultType]::ParameterName, 'The published management port [default: 9<major><minor>]. Not allowed when multiple versions are specified.')
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'The offset added to the published HTTP and management ports. Not allowed when multiple versions are specified.')
            [CompletionResult]::new('--offset', '--offset', [CompletionResultType]::ParameterName, 'The offset added to the published HTTP and management ports. Not allowed when multiple versions are specified.')
            [CompletionResult]::new('--operations', '--operations', [CompletionResultType]::ParameterName, 'A comma seperated list of operations to bootstrap the standalone server. Can be provided multiple times.')
            [CompletionResult]::new('--cli', '--cli', [CompletionResultType]::ParameterName, 'A file with operations to bootstrap the standalone server')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'wfadm;stop' {
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'The name of the standalone server [default: wfadm-sa-<major><minor>]')
            [CompletionResult]::new('--name', '--name', [CompletionResultType]::ParameterName, 'The name of the standalone server [default: wfadm-sa-<major><minor>]')
            [CompletionResult]::new('-a', '-a', [CompletionResultType]::ParameterName, 'Stop all running standalone servers. If specified with a version, stop all running standalone servers of that version.')
            [CompletionResult]::new('--all', '--all', [CompletionResultType]::ParameterName, 'Stop all running standalone servers. If specified with a version, stop all running standalone servers of that version.')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'wfadm;dc' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start a domain controller')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop a domain controller')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'wfadm;dc;start' {
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'The name of the domain controller [default: wfadm-dc-<major><minor>]. Not allowed when multiple versions are specified.')
            [CompletionResult]::new('--name', '--name', [CompletionResultType]::ParameterName, 'The name of the domain controller [default: wfadm-dc-<major><minor>]. Not allowed when multiple versions are specified.')
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'The published HTTP port [default: 8<major><minor>]. Not allowed when multiple versions are specified.')
            [CompletionResult]::new('--http', '--http', [CompletionResultType]::ParameterName, 'The published HTTP port [default: 8<major><minor>]. Not allowed when multiple versions are specified.')
            [CompletionResult]::new('-m', '-m', [CompletionResultType]::ParameterName, 'The published management port [default: 9<major><minor>]. Not allowed when multiple versions are specified.')
            [CompletionResult]::new('--management', '--management', [CompletionResultType]::ParameterName, 'The published management port [default: 9<major><minor>]. Not allowed when multiple versions are specified.')
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'The offset added to the published HTTP and management ports. Not allowed when multiple versions are specified.')
            [CompletionResult]::new('--offset', '--offset', [CompletionResultType]::ParameterName, 'The offset added to the published HTTP and management ports. Not allowed when multiple versions are specified.')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Manage servers of the domain controller. Servers are specified as a comma seperated list of <name>[:<server-group>][:<offset>][:start].  The option can be specified multiple times.  <name>          The name of the server. This part is mandatory and must be first.                 All other parts are optional. <server-group>  The name of the server group. Allowed values are ''main-server-group'' or ''msg'',                 and ''other-server-group'' or ''osg''. If not specified, ''main-server-group'' is used. <offset>        The port offset. If not specified, 100 is used from the second server onwards. start           Whether to start the server.')
            [CompletionResult]::new('--server', '--server', [CompletionResultType]::ParameterName, 'Manage servers of the domain controller. Servers are specified as a comma seperated list of <name>[:<server-group>][:<offset>][:start].  The option can be specified multiple times.  <name>          The name of the server. This part is mandatory and must be first.                 All other parts are optional. <server-group>  The name of the server group. Allowed values are ''main-server-group'' or ''msg'',                 and ''other-server-group'' or ''osg''. If not specified, ''main-server-group'' is used. <offset>        The port offset. If not specified, 100 is used from the second server onwards. start           Whether to start the server.')
            [CompletionResult]::new('--operations', '--operations', [CompletionResultType]::ParameterName, 'A comma seperated list of operations to bootstrap the domain controller. Can be provided multiple times.')
            [CompletionResult]::new('--cli', '--cli', [CompletionResultType]::ParameterName, 'A file with operations to bootstrap the domain controller')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'wfadm;dc;stop' {
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'The name of the domain controller [default: wfadm-dc-<major><minor>]')
            [CompletionResult]::new('--name', '--name', [CompletionResultType]::ParameterName, 'The name of the domain controller [default: wfadm-dc-<major><minor>]')
            [CompletionResult]::new('-a', '-a', [CompletionResultType]::ParameterName, 'Stop all running domain controllers. If specified with a version, stop all running domain controllers of that version.')
            [CompletionResult]::new('--all', '--all', [CompletionResultType]::ParameterName, 'Stop all running domain controllers. If specified with a version, stop all running domain controllers of that version.')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'wfadm;dc;help' {
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start a domain controller')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop a domain controller')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'wfadm;dc;help;start' {
            break
        }
        'wfadm;dc;help;stop' {
            break
        }
        'wfadm;dc;help;help' {
            break
        }
        'wfadm;hc' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start a host controller')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop a host controller')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'wfadm;hc;start' {
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'The name of the host controller [default: wfadm-hc-<major><minor>]. Not allowed when multiple versions are specified.')
            [CompletionResult]::new('--name', '--name', [CompletionResultType]::ParameterName, 'The name of the host controller [default: wfadm-hc-<major><minor>]. Not allowed when multiple versions are specified.')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'The name of the domain controller [default: wfadm-dc-<major><minor>]. Required if different versions are specified.')
            [CompletionResult]::new('--domain-controller', '--domain-controller', [CompletionResultType]::ParameterName, 'The name of the domain controller [default: wfadm-dc-<major><minor>]. Required if different versions are specified.')
            [CompletionResult]::new('-u', '-u', [CompletionResultType]::ParameterName, 'The username to connect to the domain controller')
            [CompletionResult]::new('--username', '--username', [CompletionResultType]::ParameterName, 'The username to connect to the domain controller')
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'The password to connect to the domain controller')
            [CompletionResult]::new('--password', '--password', [CompletionResultType]::ParameterName, 'The password to connect to the domain controller')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Manage servers of the host controller. Servers are specified as a comma seperated list of <name>[:<server-group>][:<offset>][:start].  The option can be specified multiple times.  <name>          The name of the server. This part is mandatory and must be first.                 All other parts are optional. <server-group>  The name of the server group. Allowed values are ''main-server-group'' or ''msg'',                 and ''other-server-group'' or ''osg''. If not specified, ''main-server-group'' is used. <offset>        The port offset. If not specified, 100 is used from the second server onwards. start           Whether to start the server.')
            [CompletionResult]::new('--server', '--server', [CompletionResultType]::ParameterName, 'Manage servers of the host controller. Servers are specified as a comma seperated list of <name>[:<server-group>][:<offset>][:start].  The option can be specified multiple times.  <name>          The name of the server. This part is mandatory and must be first.                 All other parts are optional. <server-group>  The name of the server group. Allowed values are ''main-server-group'' or ''msg'',                 and ''other-server-group'' or ''osg''. If not specified, ''main-server-group'' is used. <offset>        The port offset. If not specified, 100 is used from the second server onwards. start           Whether to start the server.')
            [CompletionResult]::new('--operations', '--operations', [CompletionResultType]::ParameterName, 'A comma seperated list of operations to bootstrap the host controller. Can be provided multiple times.')
            [CompletionResult]::new('--cli', '--cli', [CompletionResultType]::ParameterName, 'A file with operations to bootstrap the host controller')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'wfadm;hc;stop' {
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'The name of the host controller [default: wfadm-hc-<major><minor>]')
            [CompletionResult]::new('--name', '--name', [CompletionResultType]::ParameterName, 'The name of the host controller [default: wfadm-hc-<major><minor>]')
            [CompletionResult]::new('-a', '-a', [CompletionResultType]::ParameterName, 'Stop all running host controllers. If specified with a version, stop all running host controllers of that version.')
            [CompletionResult]::new('--all', '--all', [CompletionResultType]::ParameterName, 'Stop all running host controllers. If specified with a version, stop all running host controllers of that version.')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'wfadm;hc;help' {
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start a host controller')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop a host controller')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'wfadm;hc;help;start' {
            break
        }
        'wfadm;hc;help;stop' {
            break
        }
        'wfadm;hc;help;help' {
            break
        }
        'wfadm;topology' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start a topology')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop a topology')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'wfadm;topology;start' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'wfadm;topology;stop' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'wfadm;topology;help' {
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start a topology')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop a topology')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'wfadm;topology;help;start' {
            break
        }
        'wfadm;topology;help;stop' {
            break
        }
        'wfadm;topology;help;help' {
            break
        }
        'wfadm;images' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'wfadm;ps' {
            [CompletionResult]::new('--standalone', '--standalone', [CompletionResultType]::ParameterName, 'List standalone containers only')
            [CompletionResult]::new('--domain', '--domain', [CompletionResultType]::ParameterName, 'List domain controller and host controller containers only')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'wfadm;console' {
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'The name of the standalone server or domain controller [default: wfadm-sa|dc-<major><minor>]. Not allowed when multiple versions are specified.')
            [CompletionResult]::new('--name', '--name', [CompletionResultType]::ParameterName, 'The name of the standalone server or domain controller [default: wfadm-sa|dc-<major><minor>]. Not allowed when multiple versions are specified.')
            [CompletionResult]::new('-m', '-m', [CompletionResultType]::ParameterName, 'The published management port. Not allowed when multiple versions are specified.')
            [CompletionResult]::new('--management', '--management', [CompletionResultType]::ParameterName, 'The published management port. Not allowed when multiple versions are specified.')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'wfadm;cli' {
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'The name of the standalone server or domain controller [default: wfadm-sa|dc-<major><minor>]. Not allowed when multiple versions are specified.')
            [CompletionResult]::new('--name', '--name', [CompletionResultType]::ParameterName, 'The name of the standalone server or domain controller [default: wfadm-sa|dc-<major><minor>]. Not allowed when multiple versions are specified.')
            [CompletionResult]::new('-m', '-m', [CompletionResultType]::ParameterName, 'The published management port')
            [CompletionResult]::new('--management', '--management', [CompletionResultType]::ParameterName, 'The published management port')
            [CompletionResult]::new('-u', '-u', [CompletionResultType]::ParameterName, 'The username to connect to the CLI')
            [CompletionResult]::new('--username', '--username', [CompletionResultType]::ParameterName, 'The username to connect to the CLI')
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'The password to connect to the CLI')
            [CompletionResult]::new('--password', '--password', [CompletionResultType]::ParameterName, 'The password to connect to the CLI')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'wfadm;help' {
            [CompletionResult]::new('build', 'build', [CompletionResultType]::ParameterValue, 'Build images')
            [CompletionResult]::new('push', 'push', [CompletionResultType]::ParameterValue, 'Push images')
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start a standalone server')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop a standalone server')
            [CompletionResult]::new('dc', 'dc', [CompletionResultType]::ParameterValue, 'Start and stop a domain controller')
            [CompletionResult]::new('hc', 'hc', [CompletionResultType]::ParameterValue, 'Start and stop a host controller')
            [CompletionResult]::new('topology', 'topology', [CompletionResultType]::ParameterValue, 'Start and stop a topology defined in YAML')
            [CompletionResult]::new('images', 'images', [CompletionResultType]::ParameterValue, 'List all available standalone, domain and host controller images')
            [CompletionResult]::new('ps', 'ps', [CompletionResultType]::ParameterValue, 'List running standalone, domain and host controller containers')
            [CompletionResult]::new('console', 'console', [CompletionResultType]::ParameterValue, 'Open the management console')
            [CompletionResult]::new('cli', 'cli', [CompletionResultType]::ParameterValue, 'Connect to the CLI')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'wfadm;help;build' {
            break
        }
        'wfadm;help;push' {
            break
        }
        'wfadm;help;start' {
            break
        }
        'wfadm;help;stop' {
            break
        }
        'wfadm;help;dc' {
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start a domain controller')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop a domain controller')
            break
        }
        'wfadm;help;dc;start' {
            break
        }
        'wfadm;help;dc;stop' {
            break
        }
        'wfadm;help;hc' {
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start a host controller')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop a host controller')
            break
        }
        'wfadm;help;hc;start' {
            break
        }
        'wfadm;help;hc;stop' {
            break
        }
        'wfadm;help;topology' {
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start a topology')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop a topology')
            break
        }
        'wfadm;help;topology;start' {
            break
        }
        'wfadm;help;topology;stop' {
            break
        }
        'wfadm;help;images' {
            break
        }
        'wfadm;help;ps' {
            break
        }
        'wfadm;help;console' {
            break
        }
        'wfadm;help;cli' {
            break
        }
        'wfadm;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
