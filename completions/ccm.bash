# Bash completion for Claude Config Manager (ccm)

_ccm_completion() {
    local cur prev words cword
    _init_completion || return

    case ${prev} in
        ccm)
            COMPREPLY=($(compgen -W "config history mcp project search help" -- "${cur}"))
            ;;
        config)
            COMPREPLY=($(compgen -W "get set diff import export help" -- "${cur}"))
            ;;
        history)
            COMPREPLY=($(compgen -W "list restore help" -- "${cur}"))
            ;;
        mcp)
            COMPREPLY=($(compgen -W "list add remove enable disable show help" -- "${cur}"))
            ;;
        project)
            COMPREPLY=($(compgen -W "scan list config help" -- "${cur}"))
            ;;
        get|diff)
            COMPREPLY=($(compgen -W "--output --project --help" -- "${cur}"))
            ;;
        set)
            COMPREPLY=($(compgen -W "--project --scope --help" -- "${cur}"))
            ;;
        import|export)
            COMPREPLY=($(compgen -W "--format --project --help" -- "${cur}"))
            ;;
        list)
            COMPREPLY=($(compgen -W "--scope --output --project --help" -- "${cur}"))
            ;;
        add)
            COMPREPLY=($(compgen -W "--command --args --env --scope --help" -- "${cur}"))
            ;;
        remove|enable|disable|show)
            COMPREPLY=($(compgen -W "--scope --project --help" -- "${cur}"))
            ;;
        scan)
            COMPREPLY=($(compgen -W "--path --depth --ignore --help" -- "${cur}"))
            ;;
        restore)
            COMPREPLY=($(compgen -W "--project --help" -- "${cur}"))
            ;;
        search)
            COMPREPLY=($(compgen -W "--key --value --case-sensitive --regex --scope --project --help" -- "${cur}"))
            ;;
        --output)
            COMPREPLY=($(compgen -W "table json" -- "${cur}"))
            ;;
        --scope)
            COMPREPLY=($(compgen -W "global project" -- "${cur}"))
            ;;
        --format)
            COMPREPLY=($(compgen -W "json toml" -- "${cur}"))
            ;;
        *)
            COMPREPLY=($(compgen -W "--help --verbose" -- "${cur}"))
            ;;
    esac
} &&

complete -F _ccm_completion ccm
