complete -c blaze -n "__fish_use_subcommand" -s h -l help -d 'Print help'
complete -c blaze -n "__fish_use_subcommand" -f -a "install" -d 'install a new NodeJS package'
complete -c blaze -n "__fish_use_subcommand" -f -a "init" -d 'initialize a new NodeJS project'
complete -c blaze -n "__fish_use_subcommand" -f -a "version" -d 'Print the version'
complete -c blaze -n "__fish_use_subcommand" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c blaze -n "__fish_seen_subcommand_from install" -s h -l help -d 'Print help'
complete -c blaze -n "__fish_seen_subcommand_from init" -s h -l help -d 'Print help'
complete -c blaze -n "__fish_seen_subcommand_from version" -s h -l help -d 'Print help'
complete -c blaze -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from install; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from version; and not __fish_seen_subcommand_from help" -f -a "install" -d 'install a new NodeJS package'
complete -c blaze -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from install; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from version; and not __fish_seen_subcommand_from help" -f -a "init" -d 'initialize a new NodeJS project'
complete -c blaze -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from install; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from version; and not __fish_seen_subcommand_from help" -f -a "version" -d 'Print the version'
complete -c blaze -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from install; and not __fish_seen_subcommand_from init; and not __fish_seen_subcommand_from version; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
