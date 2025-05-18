use clap::builder::Styles;
use clap::builder::styling::{AnsiColor, Effects};
use clap::{Arg, ArgAction, Command, crate_name, crate_version, value_parser};

pub fn build_app() -> Command {
    Command::new(crate_name!())
        .version(crate_version!())
        .about("Command line tool to build and run WildFly containers in different versions and operation modes.")
        .styles(Styles::styled()
            .header(AnsiColor::Green.on_default() | Effects::BOLD)
            .usage(AnsiColor::Green.on_default() | Effects::BOLD)
            .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
            .placeholder(AnsiColor::Cyan.on_default()))
        .propagate_version(true)
        .subcommand_required(true)

        // build
        .subcommand(Command::new("build")
            .about("Build images")
            .arg(Arg::new("wildfly-version")
                .required(true)
                .help("A single WildFly version or version range"))
            .arg(Arg::new("username")
                .short('u')
                .long("username")
                .default_value("admin")
                .help("The username of the management user"))
            .arg(Arg::new("password")
                .short('p')
                .long("password")
                .default_value("admin")
                .help("The password of the management user"))
            .arg(Arg::new("standalone")
                .long("standalone")
                .action(ArgAction::SetTrue)
                .help("Build standalone images only"))
            .arg(Arg::new("domain")
                .long("domain")
                .action(ArgAction::SetTrue)
                .help("Build domain controller and host controller images only"))
            .arg(Arg::new("chunks")
                .long("chunks")
                .value_parser(value_parser!(u16))
                .help("Build the images in chunks of this size. If not specified, the images are built in one go.")))

        // push
        .subcommand(Command::new("push")
            .about("Push images")
            .arg(Arg::new("wildfly-version")
                .required(true)
                .help("A single WildFly version or version range"))
            .arg(Arg::new("standalone")
                .long("standalone")
                .action(ArgAction::SetTrue)
                .help("Push standalone images only"))
            .arg(Arg::new("domain")
                .long("domain")
                .action(ArgAction::SetTrue)
                .help("Push domain controller and host controller images only"))
            .arg(Arg::new("chunks")
                .long("chunks")
                .value_parser(value_parser!(u16))
                .help("Push the images in chunks of this size. If not specified, the images are pushed in one go.")))

        // standalone start
        .subcommand(Command::new("start")
            .about("Start a standalone server")
            .arg(Arg::new("wildfly-version")
                .index(1)
                .required(true)
                .help("A single WildFly version or version range"))
            .arg(Arg::new("wildfly-parameters")
                .index(2)
                .last(true)
                .num_args(0..)
                .required(false)
                .help("Parameters passed to the standalone server"))
            .arg(Arg::new("name")
                .short('n')
                .long("name")
                .help("The name of the standalone server [default: wfadm-sa-<major><minor>].
Not allowed when multiple versions are specified."))
            .arg(Arg::new("http")
                .short('p')
                .long("http")
                .value_parser(value_parser!(u16))
                .help("The published HTTP port [default: 8<major><minor>].
Not allowed when multiple versions are specified."))
            .arg(Arg::new("management")
                .short('m')
                .long("management")
                .value_parser(value_parser!(u16))
                .help("The published management port [default: 9<major><minor>].
Not allowed when multiple versions are specified."))
            .arg(Arg::new("offset")
                .short('o')
                .long("offset")
                .value_parser(value_parser!(u16).range(1..))
                .help("The offset added to the published HTTP and management ports.
Not allowed when multiple versions are specified."))
            .arg(Arg::new("operations")
                .long("operations")
                .action(ArgAction::Append)
                .help("A comma seperated list of operations to bootstrap the standalone server.
Can be provided multiple times."))
            .arg(Arg::new("cli")
                .long("cli")
                .help("A file with operations to bootstrap the standalone server")))

        // standalone stop
        .subcommand(Command::new("stop")
            .about("Stop a standalone server")
            .arg(Arg::new("wildfly-version")
                .required_unless_present("all")
                .help("A single WildFly version or version range"))
            .arg(Arg::new("name")
                .short('n')
                .long("name")
                .help("The name of the standalone server [default: wfadm-sa-<major><minor>]"))
            .arg(Arg::new("all")
                .short('a')
                .long("all")
                .action(ArgAction::SetTrue)
                .help("Stop all running standalone servers. If specified with a version,
stop all running standalone servers of that version.")))

        // domain controller
        .subcommand(Command::new("dc")
            .about("Start and stop a domain controller")

            // start
            .subcommand(Command::new("start")
                .about("Start a domain controller")
                .arg(Arg::new("wildfly-version")
                    .index(1)
                    .required(true)
                    .help("A single WildFly version or version range"))
                .arg(Arg::new("wildfly-parameters")
                    .index(2)
                    .last(true)
                    .num_args(0..)
                    .required(false)
                    .help("Parameters passed to the domain controller"))
                .arg(Arg::new("name")
                    .short('n')
                    .long("name")
                    .help("The name of the domain controller [default: wfadm-dc-<major><minor>].
Not allowed when multiple versions are specified."))
                .arg(Arg::new("http")
                    .short('p')
                    .long("http")
                    .value_parser(value_parser!(u16))
                    .help("The published HTTP port [default: 8<major><minor>].
Not allowed when multiple versions are specified."))
                .arg(Arg::new("management")
                    .short('m')
                    .long("management")
                    .value_parser(value_parser!(u16))
                    .help("The published management port [default: 9<major><minor>].
Not allowed when multiple versions are specified."))
                .arg(Arg::new("offset")
                    .short('o')
                    .long("offset")
                    .value_parser(value_parser!(u16).range(1..))
                    .help("The offset added to the published HTTP and management ports.
Not allowed when multiple versions are specified."))
                .arg(Arg::new("server")
                    .short('s')
                    .long("server")
                    .action(ArgAction::Append)
                    .help("Manage servers of the domain controller.
Servers are specified as a comma seperated list of <name>[:<server-group>][:<offset>][:start]. 
The option can be specified multiple times. 
<name>          The name of the server. This part is mandatory and must be first.
                All other parts are optional.
<server-group>  The name of the server group. Allowed values are 'main-server-group' or 'msg',
                and 'other-server-group' or 'osg'. If not specified, 'main-server-group' is used.
<offset>        The port offset. If not specified, 100 is used from the second server onwards.
start           Whether to start the server."))
                .arg(Arg::new("operations")
                    .long("operations")
                    .action(ArgAction::Append)
                    .help("A comma seperated list of operations to bootstrap the domain controller.
Can be provided multiple times."))
                .arg(Arg::new("cli")
                    .long("cli")
                    .help("A file with operations to bootstrap the domain controller")))

            // stop
            .subcommand(Command::new("stop")
                .about("Stop a domain controller")
                .arg(Arg::new("wildfly-version")
                    .required_unless_present("all")
                    .help("A single WildFly version or version range"))
                .arg(Arg::new("name")
                    .short('n')
                    .long("name")
                    .help("The name of the domain controller [default: wfadm-dc-<major><minor>]"))
                .arg(Arg::new("all")
                    .short('a')
                    .long("all")
                    .action(ArgAction::SetTrue)
                    .help("Stop all running domain controllers. If specified with a version,
stop all running domain controllers of that version."))))

        // host controller
        .subcommand(Command::new("hc")
            .about("Start and stop a host controller")

            // start
            .subcommand(Command::new("start")
                .about("Start a host controller")
                .arg(Arg::new("wildfly-version")
                    .index(1)
                    .required(true)
                    .help("A single WildFly version or version range"))
                .arg(Arg::new("wildfly-parameters")
                    .index(2)
                    .last(true)
                    .num_args(0..)
                    .required(false)
                    .help("Parameters passed to the domain controller"))
                .arg(Arg::new("name")
                    .short('n')
                    .long("name")
                    .help("The name of the host controller [default: wfadm-hc-<major><minor>].
Not allowed when multiple versions are specified."))
                .arg(Arg::new("domain-controller")
                    .short('d')
                    .long("domain-controller")
                    .help("The name of the domain controller [default: wfadm-dc-<major><minor>].
Required if different versions are specified."))
                .arg(Arg::new("username")
                    .short('u')
                    .long("username")
                    .default_value("admin")
                    .help("The username to connect to the domain controller"))
                .arg(Arg::new("password")
                    .short('p')
                    .long("password")
                    .default_value("admin")
                    .help("The password to connect to the domain controller"))
                .arg(Arg::new("server")
                    .short('s')
                    .long("server")
                    .action(ArgAction::Append)
                    .help("Manage servers of the host controller.
Servers are specified as a comma seperated list of <name>[:<server-group>][:<offset>][:start]. 
The option can be specified multiple times. 
<name>          The name of the server. This part is mandatory and must be first.
                All other parts are optional.
<server-group>  The name of the server group. Allowed values are 'main-server-group' or 'msg',
                and 'other-server-group' or 'osg'. If not specified, 'main-server-group' is used.
<offset>        The port offset. If not specified, 100 is used from the second server onwards.
start           Whether to start the server."))
                .arg(Arg::new("operations")
                    .long("operations")
                    .action(ArgAction::Append)
                    .help("A comma seperated list of operations to bootstrap the host controller.
Can be provided multiple times."))
                .arg(Arg::new("cli")
                    .long("cli")
                    .help("A file with operations to bootstrap the host controller")))

            // stop
            .subcommand(Command::new("stop")
                .about("Stop a host controller")
                .arg(Arg::new("wildfly-version")
                    .required_unless_present("all")
                    .help("A single WildFly version or version range"))
                .arg(Arg::new("name")
                    .short('n')
                    .long("name")
                    .help("The name of the host controller [default: wfadm-hc-<major><minor>]"))
                .arg(Arg::new("all")
                    .short('a')
                    .long("all")
                    .action(ArgAction::SetTrue)
                    .help("Stop all running host controllers. If specified with a version,
stop all running host controllers of that version."))))

        // topology
        .subcommand(Command::new("topology")
            .about("Start and stop a topology defined in YAML")

            // start
            .subcommand(Command::new("start")
                .about("Start a topology")
                .arg(Arg::new("setup")
                    .required(true)
                    .help("The topology setup")))

            // stop
            .subcommand(Command::new("stop")
                .about("Stop a topology")
                .arg(Arg::new("setup")
                    .required(true)
                    .help("The topology setup"))))

        // images
        .subcommand(Command::new("images")
            .about("List all available standalone, domain and host controller images"))

        // ps
        .subcommand(Command::new("ps")
            .about("List running standalone, domain and host controller containers")
            .arg(Arg::new("standalone")
                .long("standalone")
                .action(ArgAction::SetTrue)
                .help("List standalone containers only"))
            .arg(Arg::new("domain")
                .long("domain")
                .action(ArgAction::SetTrue)
                .help("List domain controller and host controller containers only")))

        // console
        .subcommand(Command::new("console")
            .about("Open the management console")
            .arg(Arg::new("wildfly-version")
                .help("A single WildFly version or version range.
If omitted the console is opened for all running standalone and domain controller containers."))
            .arg(Arg::new("name")
                .short('n')
                .long("name")
                .help("The name of the standalone server or domain controller [default: wfadm-sa|dc-<major><minor>].
Not allowed when multiple versions are specified."))
            .arg(Arg::new("management")
                .short('m')
                .long("management")
                .value_parser(value_parser!(u16))
                .conflicts_with("name")
                .help("The published management port. Not allowed when multiple versions are specified.")))

        // cli
        .subcommand(Command::new("cli")
            .about("Connect to the CLI")
            .arg(Arg::new("wildfly-version")
                .index(1)
                .help("A single WildFly version.
Can be omitted if only one standalone or domain controller is running."))
            .arg(Arg::new("cli-parameters")
                .index(2)
                .last(true)
                .num_args(0..)
                .required(false)
                .help("Parameters passed to the CLI"))
            .arg(Arg::new("name")
                .short('n')
                .long("name")
                .help("The name of the standalone server or domain controller [default: wfadm-sa|dc-<major><minor>].
Not allowed when multiple versions are specified."))
            .arg(Arg::new("management")
                .short('m')
                .long("management")
                .value_parser(value_parser!(u16))
                .conflicts_with("name")
                .help("The published management port"))
            .arg(Arg::new("username")
                .short('u')
                .long("username")
                .default_value("admin")
                .help("The username to connect to the CLI"))
            .arg(Arg::new("password")
                .short('p')
                .long("password")
                .default_value("admin")
                .help("The password to connect to the CLI")))
}
