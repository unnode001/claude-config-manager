# Zsh completion for Claude Config Manager (ccm)

_ccm() {
    local -a commands subcommands
    commands=(
        'config:Configuration management commands'
        'history:History and backup management commands'
        'mcp:MCP server management commands'
        'project:Project discovery and management commands'
        'search:Search configuration values'
        'help:Show help information'
    )

    local -a config_subcommands
    config_subcommands=(
        'get:View configuration'
        'set:Set configuration value'
        'diff:Compare configurations'
        'import:Import from file'
        'export:Export to file'
    )

    local -a history_subcommands
    history_subcommands=(
        'list:List backups'
        'restore:Restore backup'
    )

    local -a mcp_subcommands
    mcp_subcommands=(
        'list:List servers'
        'add:Add new server'
        'remove:Remove server'
        'enable:Enable server'
        'disable:Disable server'
        'show:Show server details'
    )

    local -a project_subcommands
    project_subcommands=(
        'scan:Scan for projects'
        'list:List projects'
        'config:Show project config'
    )

    local -a output_formats
    output_formats=('table' 'json')

    local -a scopes
    scopes=('global' 'project')

    local -a formats
    formats=('json' 'toml')

    local curcontext="$curcontext" state line
    _arguments -C \
        '1: :->command' \
        '*::arg:->args'

    case $state in
        command)
            _describe 'command' commands
            ;;
        args)
            case ${words[2]} in
                config)
                    _values 'config subcommands' $config_subcommands
                    ;;
                history)
                    _values 'history subcommands' $history_subcommands
                    ;;
                mcp)
                    _values 'mcp subcommands' $mcp_subcommands
                    ;;
                project)
                    _values 'project subcommands' $project_subcommands
                    ;;
            esac
            ;;
    esac
}

_ccm
