
use builtin;
use str;

set edit:completion:arg-completer[waco] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'waco'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'waco'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand build 'Build images'
            cand push 'Push images'
            cand start 'Start a standalone server'
            cand stop 'Stop a standalone server'
            cand dc 'Start and stop a domain controller'
            cand hc 'Start and stop a host controller'
            cand topology 'Start and stop a topology defined in YAML'
            cand images 'List all available standalone, domain and host controller images'
            cand ps 'List running standalone, domain and host controller containers'
            cand console 'Open the management console'
            cand cli 'Connect to the CLI'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'waco;build'= {
            cand -u 'The username of the management user'
            cand --username 'The username of the management user'
            cand -p 'The password of the management user'
            cand --password 'The password of the management user'
            cand --chunks 'Build the images in chunks of this size. If not specified, the images are built in one go.'
            cand --standalone 'Build standalone images only'
            cand --domain 'Build domain controller and host controller images only'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'waco;push'= {
            cand --chunks 'Push the images in chunks of this size. If not specified, the images are pushed in one go.'
            cand --standalone 'Push standalone images only'
            cand --domain 'Push domain controller and host controller images only'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'waco;start'= {
            cand -n 'The name of the standalone server [default: waco-sa-<major><minor>]. Not allowed when multiple versions are specified.'
            cand --name 'The name of the standalone server [default: waco-sa-<major><minor>]. Not allowed when multiple versions are specified.'
            cand -p 'The published HTTP port [default: 8<major><minor>]. Not allowed when multiple versions are specified.'
            cand --http 'The published HTTP port [default: 8<major><minor>]. Not allowed when multiple versions are specified.'
            cand -m 'The published management port [default: 9<major><minor>]. Not allowed when multiple versions are specified.'
            cand --management 'The published management port [default: 9<major><minor>]. Not allowed when multiple versions are specified.'
            cand -o 'The offset added to the published HTTP and management ports. Not allowed when multiple versions are specified.'
            cand --offset 'The offset added to the published HTTP and management ports. Not allowed when multiple versions are specified.'
            cand --operations 'A comma seperated list of operations to bootstrap the standalone server. Can be provided multiple times.'
            cand --cli 'A file with operations to bootstrap the standalone server'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'waco;stop'= {
            cand -n 'The name of the standalone server [default: waco-sa-<major><minor>]'
            cand --name 'The name of the standalone server [default: waco-sa-<major><minor>]'
            cand -a 'Stop all running standalone servers. If specified with a version, stop all running standalone servers of that version.'
            cand --all 'Stop all running standalone servers. If specified with a version, stop all running standalone servers of that version.'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'waco;dc'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand start 'Start a domain controller'
            cand stop 'Stop a domain controller'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'waco;dc;start'= {
            cand -n 'The name of the domain controller [default: waco-dc-<major><minor>]. Not allowed when multiple versions are specified.'
            cand --name 'The name of the domain controller [default: waco-dc-<major><minor>]. Not allowed when multiple versions are specified.'
            cand -p 'The published HTTP port [default: 8<major><minor>]. Not allowed when multiple versions are specified.'
            cand --http 'The published HTTP port [default: 8<major><minor>]. Not allowed when multiple versions are specified.'
            cand -m 'The published management port [default: 9<major><minor>]. Not allowed when multiple versions are specified.'
            cand --management 'The published management port [default: 9<major><minor>]. Not allowed when multiple versions are specified.'
            cand -o 'The offset added to the published HTTP and management ports. Not allowed when multiple versions are specified.'
            cand --offset 'The offset added to the published HTTP and management ports. Not allowed when multiple versions are specified.'
            cand -s 'Manage servers of the domain controller. Servers are specified as a comma seperated list of <name>[:<server-group>][:<offset>][:start].  The option can be specified multiple times.  <name>          The name of the server. This part is mandatory and must be first.                 All other parts are optional. <server-group>  The name of the server group. Allowed values are ''main-server-group'' or ''msg'',                 and ''other-server-group'' or ''osg''. If not specified, ''main-server-group'' is used. <offset>        The port offset. If not specified, 100 is used from the second server onwards. start           Whether to start the server.'
            cand --server 'Manage servers of the domain controller. Servers are specified as a comma seperated list of <name>[:<server-group>][:<offset>][:start].  The option can be specified multiple times.  <name>          The name of the server. This part is mandatory and must be first.                 All other parts are optional. <server-group>  The name of the server group. Allowed values are ''main-server-group'' or ''msg'',                 and ''other-server-group'' or ''osg''. If not specified, ''main-server-group'' is used. <offset>        The port offset. If not specified, 100 is used from the second server onwards. start           Whether to start the server.'
            cand --operations 'A comma seperated list of operations to bootstrap the domain controller. Can be provided multiple times.'
            cand --cli 'A file with operations to bootstrap the domain controller'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'waco;dc;stop'= {
            cand -n 'The name of the domain controller [default: waco-dc-<major><minor>]'
            cand --name 'The name of the domain controller [default: waco-dc-<major><minor>]'
            cand -a 'Stop all running domain controllers. If specified with a version, stop all running domain controllers of that version.'
            cand --all 'Stop all running domain controllers. If specified with a version, stop all running domain controllers of that version.'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'waco;dc;help'= {
            cand start 'Start a domain controller'
            cand stop 'Stop a domain controller'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'waco;dc;help;start'= {
        }
        &'waco;dc;help;stop'= {
        }
        &'waco;dc;help;help'= {
        }
        &'waco;hc'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand start 'Start a host controller'
            cand stop 'Stop a host controller'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'waco;hc;start'= {
            cand -n 'The name of the host controller [default: waco-hc-<major><minor>]. Not allowed when multiple versions are specified.'
            cand --name 'The name of the host controller [default: waco-hc-<major><minor>]. Not allowed when multiple versions are specified.'
            cand -d 'The name of the domain controller [default: waco-dc-<major><minor>]. Required if different versions are specified.'
            cand --domain-controller 'The name of the domain controller [default: waco-dc-<major><minor>]. Required if different versions are specified.'
            cand -u 'The username to connect to the domain controller'
            cand --username 'The username to connect to the domain controller'
            cand -p 'The password to connect to the domain controller'
            cand --password 'The password to connect to the domain controller'
            cand -s 'Manage servers of the host controller. Servers are specified as a comma seperated list of <name>[:<server-group>][:<offset>][:start].  The option can be specified multiple times.  <name>          The name of the server. This part is mandatory and must be first.                 All other parts are optional. <server-group>  The name of the server group. Allowed values are ''main-server-group'' or ''msg'',                 and ''other-server-group'' or ''osg''. If not specified, ''main-server-group'' is used. <offset>        The port offset. If not specified, 100 is used from the second server onwards. start           Whether to start the server.'
            cand --server 'Manage servers of the host controller. Servers are specified as a comma seperated list of <name>[:<server-group>][:<offset>][:start].  The option can be specified multiple times.  <name>          The name of the server. This part is mandatory and must be first.                 All other parts are optional. <server-group>  The name of the server group. Allowed values are ''main-server-group'' or ''msg'',                 and ''other-server-group'' or ''osg''. If not specified, ''main-server-group'' is used. <offset>        The port offset. If not specified, 100 is used from the second server onwards. start           Whether to start the server.'
            cand --operations 'A comma seperated list of operations to bootstrap the host controller. Can be provided multiple times.'
            cand --cli 'A file with operations to bootstrap the host controller'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'waco;hc;stop'= {
            cand -n 'The name of the host controller [default: waco-hc-<major><minor>]'
            cand --name 'The name of the host controller [default: waco-hc-<major><minor>]'
            cand -a 'Stop all running host controllers. If specified with a version, stop all running host controllers of that version.'
            cand --all 'Stop all running host controllers. If specified with a version, stop all running host controllers of that version.'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'waco;hc;help'= {
            cand start 'Start a host controller'
            cand stop 'Stop a host controller'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'waco;hc;help;start'= {
        }
        &'waco;hc;help;stop'= {
        }
        &'waco;hc;help;help'= {
        }
        &'waco;topology'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand start 'Start a topology'
            cand stop 'Stop a topology'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'waco;topology;start'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'waco;topology;stop'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'waco;topology;help'= {
            cand start 'Start a topology'
            cand stop 'Stop a topology'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'waco;topology;help;start'= {
        }
        &'waco;topology;help;stop'= {
        }
        &'waco;topology;help;help'= {
        }
        &'waco;images'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'waco;ps'= {
            cand --standalone 'List standalone containers only'
            cand --domain 'List domain controller and host controller containers only'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'waco;console'= {
            cand -n 'The name of the standalone server or domain controller [default: waco-sa|dc-<major><minor>]. Not allowed when multiple versions are specified.'
            cand --name 'The name of the standalone server or domain controller [default: waco-sa|dc-<major><minor>]. Not allowed when multiple versions are specified.'
            cand -m 'The published management port. Not allowed when multiple versions are specified.'
            cand --management 'The published management port. Not allowed when multiple versions are specified.'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'waco;cli'= {
            cand -n 'The name of the standalone server or domain controller [default: waco-sa|dc-<major><minor>]. Not allowed when multiple versions are specified.'
            cand --name 'The name of the standalone server or domain controller [default: waco-sa|dc-<major><minor>]. Not allowed when multiple versions are specified.'
            cand -m 'The published management port'
            cand --management 'The published management port'
            cand -u 'The username to connect to the CLI'
            cand --username 'The username to connect to the CLI'
            cand -p 'The password to connect to the CLI'
            cand --password 'The password to connect to the CLI'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'waco;help'= {
            cand build 'Build images'
            cand push 'Push images'
            cand start 'Start a standalone server'
            cand stop 'Stop a standalone server'
            cand dc 'Start and stop a domain controller'
            cand hc 'Start and stop a host controller'
            cand topology 'Start and stop a topology defined in YAML'
            cand images 'List all available standalone, domain and host controller images'
            cand ps 'List running standalone, domain and host controller containers'
            cand console 'Open the management console'
            cand cli 'Connect to the CLI'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'waco;help;build'= {
        }
        &'waco;help;push'= {
        }
        &'waco;help;start'= {
        }
        &'waco;help;stop'= {
        }
        &'waco;help;dc'= {
            cand start 'Start a domain controller'
            cand stop 'Stop a domain controller'
        }
        &'waco;help;dc;start'= {
        }
        &'waco;help;dc;stop'= {
        }
        &'waco;help;hc'= {
            cand start 'Start a host controller'
            cand stop 'Stop a host controller'
        }
        &'waco;help;hc;start'= {
        }
        &'waco;help;hc;stop'= {
        }
        &'waco;help;topology'= {
            cand start 'Start a topology'
            cand stop 'Stop a topology'
        }
        &'waco;help;topology;start'= {
        }
        &'waco;help;topology;stop'= {
        }
        &'waco;help;images'= {
        }
        &'waco;help;ps'= {
        }
        &'waco;help;console'= {
        }
        &'waco;help;cli'= {
        }
        &'waco;help;help'= {
        }
    ]
    $completions[$command]
}
