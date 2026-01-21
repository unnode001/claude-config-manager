# PowerShell completion for Claude Config Manager (ccm)

using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName ccm -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commands = @('config', 'history', 'mcp', 'project', 'search', 'help')

    $subcommands = @{
        'config' = @('get', 'set', 'diff', 'import', 'export')
        'history' = @('list', 'restore')
        'mcp' = @('list', 'add', 'remove', 'enable', 'disable', 'show')
        'project' = @('scan', 'list', 'config')
    }

    $options = @{
        'get' = @('--output', '--project')
        'set' = @('--project', '--scope')
        'list' = @('--scope', '--output', '--project')
        'add' = @('--command', '--args', '--env', '--scope')
        'scan' = @('--path', '--depth', '--ignore')
        'search' = @('--key', '--value', '--case-sensitive', '--regex', '--scope')
    }

    $commandElements = $commandAst.CommandElements
    $command = if ($commandElements.Count -gt 1) { $commandElements[1].Extent.Text } else { $null }

    if ($command -in $commands) {
        if ($commandElements.Count -le 2) {
            $subcommands[$command] | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
                [CompletionResult]::new($_, $_, [CompletionResultType]::ParameterValue, $_)
            }
        } else {
            $subcommand = $commandElements[3].Extent.Text
            if ($options.ContainsKey($subcommand)) {
                $options[$subcommand] | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
                    [CompletionResult]::new($_, $_, [CompletionResultType]::ParameterName, $_)
                }
            }
        }
    } else {
        $commands | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [CompletionResult]::new($_, $_, [CompletionResultType]::Command, $_)
        }

        @('--help', '--verbose') | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [CompletionResult]::new($_, $_, [CompletionResultType]::ParameterName, $_)
        }
    }
}
