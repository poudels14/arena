#!/usr/bin/env bash
# credit: bun.sh
set -euo pipefail

if [[ ${OS:-} = Windows_NT ]]; then
    echo 'error: Please install portal using Windows Subsystem for Linux'
    exit 1
fi

# Reset
Color_Off=''

# Regular Colors
Red=''
Green=''
Dim='' # White

# Bold
Bold_White=''
Bold_Green=''

if [[ -t 1 ]]; then
    # Reset
    Color_Off='\033[0m' # Text Reset

    # Regular Colors
    Red='\033[0;31m'   # Red
    Green='\033[0;32m' # Green
    Dim='\033[0;2m'    # White

    # Bold
    Bold_Green='\033[1;32m' # Bold Green
    Bold_White='\033[1m'    # Bold White
fi

error() {
    echo -e "${Red}error${Color_Off}:" "$@" >&2
    exit 1
}

info() {
    echo -e "${Dim}$@ ${Color_Off}"
}

info_bold() {
    echo -e "${Bold_White}$@ ${Color_Off}"
}

success() {
    echo -e "${Green}$@ ${Color_Off}"
}

command -v unzip >/dev/null ||
    error 'unzip is required to install portal (see: https://github.com/poudels14/portal-release)'

if [[ $# -gt 0 ]]; then
    error 'No install arguments allowed.'
fi

case $(uname -ms) in
'Darwin x86_64')
    target=darwin-x64
    ;;
'Darwin arm64')
    target=darwin-aarch64
    ;;
'Linux aarch64' | 'Linux arm64')
    target=linux-aarch64
    ;;
'Linux x86_64' | *)
    target=linux-x64
    ;;
esac

if [[ $target = darwin-x64 ]]; then
    # Is this process running in Rosetta?
    # redirect stderr to devnull to avoid error message when not running in Rosetta
    if [[ $(sysctl -n sysctl.proc_translated 2>/dev/null) = 1 ]]; then
        target=darwin-aarch64
        info "Your shell is running in Rosetta 2. Downloading portal for $target instead"
    fi
fi

GITHUB=${GITHUB-"https://github.com"}

github_repo="$GITHUB/poudels14/portal-release"

if [[ $target = darwin-x64 ]]; then
    # If AVX2 isn't supported, use the -baseline build
    if [[ $(sysctl -a | grep machdep.cpu | grep AVX2) == '' ]]; then
        # target=darwin-x64-baseline
        echo 'error: Unsupported platform [code: darwin-x64-no-avx]'
        exit 1
    fi
fi

if [[ $target = linux-x64 ]]; then
    # If AVX2 isn't supported, use the -baseline build
    if [[ $(cat /proc/cpuinfo | grep avx2) = '' ]]; then
        # target=linux-x64-baseline
        echo 'error: Unsupported platform [code: linux-x64-no-avx]'
        exit 1
    fi
fi

exe_name=portal

if [[ $# = 0 ]]; then
    portal_uri=$github_repo/releases/latest/download/portal-$target.zip
else
    portal_uri=$github_repo/releases/download/$1/portal-$target.zip
fi

install_env=PORTAL_INSTALL
bin_env=\$$install_env/bin

install_dir=${!install_env:-$HOME/.portal}
bin_dir=$install_dir/bin
exe=$bin_dir/$exe_name
tmp_dir=$install_dir/tmp
tmp_zip=$tmp_dir/$exe_name.zip

# uncomment for test
# portal_uri="http://0.0.0.0:8000/portal-$target.zip"

if [[ ! -d $tmp_dir ]]; then
    mkdir -p "$tmp_dir" ||
        error "Failed to create temporary directory \"$tmp_dir\""
fi

# error "nice"
curl --fail --location --progress-bar --output "$tmp_zip" "$portal_uri" ||
    error "Failed to download portal from \"$portal_uri\""

# zip should have following dir structure
#   - /portal-$target/bin/portal
#   - /portal-$target/lib/$thirdparty-libs
unzip -oqd "$tmp_dir" "$tmp_zip" ||
    error 'Failed to extract portal'

mv "$tmp_dir/portal-$target"/* "$install_dir/" ||
    error 'Failed to move extracted portal to destination'

chmod +x "$exe" ||
    error 'Failed to set permissions on portal executable'

rm -r "$tmp_dir"

tildify() {
    if [[ $1 = $HOME/* ]]; then
        local replacement=\~/

        echo "${1/$HOME\//$replacement}"
    else
        echo "$1"
    fi
}

success "portal was installed successfully to $Bold_Green$(tildify "$exe")"

# if command -v portal >/dev/null; then
#     # Install completions, but we don't care if it fails
#     IS_BUN_AUTO_UPDATE=true $exe completions &>/dev/null || :

#     echo "Run 'portal --help' to get started"
#     exit
# fi

# refresh_command=''

tilde_bin_dir=$(tildify "$bin_dir")
quoted_install_dir=\"${install_dir//\"/\\\"}\"

if [[ $quoted_install_dir = \"$HOME/* ]]; then
    quoted_install_dir=${quoted_install_dir/$HOME\//\$HOME/}
fi

echo

case $(basename "$SHELL") in
fish)
    # Install completions, but we don't care if it fails
    # IS_BUN_AUTO_UPDATE=true SHELL=fish $exe completions &>/dev/null || :

    commands=(
        "set --export $install_env $quoted_install_dir"
        "set --export PATH $bin_env \$PATH"
    )

    fish_config=$HOME/.config/fish/config.fish
    tilde_fish_config=$(tildify "$fish_config")

    if [[ -w $fish_config ]]; then
        {
            echo -e '\n# portal'

            for command in "${commands[@]}"; do
                echo "$command"
            done
        } >>"$fish_config"

        info "Added \"$tilde_bin_dir\" to \$PATH in \"$tilde_fish_config\""

        refresh_command="source $tilde_fish_config"
    else
        echo "Manually add the directory to $tilde_fish_config (or similar):"

        for command in "${commands[@]}"; do
            info_bold "  $command"
        done
    fi
    ;;
zsh)
    # Install completions, but we don't care if it fails
    # IS_BUN_AUTO_UPDATE=true SHELL=zsh $exe completions &>/dev/null || :

    commands=(
        "export $install_env=$quoted_install_dir"
        "export PATH=\"$bin_env:\$PATH\""
    )

    zsh_config=$HOME/.zshrc
    tilde_zsh_config=$(tildify "$zsh_config")

    if [[ -w $zsh_config ]]; then
        {
            echo -e '\n# portal'

            for command in "${commands[@]}"; do
                echo "$command"
            done
        } >>"$zsh_config"

        info "Added \"$tilde_bin_dir\" to \$PATH in \"$tilde_zsh_config\""

        refresh_command="exec $SHELL"
    else
        echo "Manually add the directory to $tilde_zsh_config (or similar):"

        for command in "${commands[@]}"; do
            info_bold "  $command"
        done
    fi
    ;;
bash)
    # Install completions, but we don't care if it fails
    # IS_BUN_AUTO_UPDATE=true SHELL=bash $exe completions &>/dev/null || :

    commands=(
        "export $install_env=$quoted_install_dir"
        "export PATH=$bin_env:\$PATH"
    )

    bash_configs=(
        "$HOME/.bashrc"
        "$HOME/.bash_profile"
    )

    if [[ ${XDG_CONFIG_HOME:-} ]]; then
        bash_configs+=(
            "$XDG_CONFIG_HOME/.bash_profile"
            "$XDG_CONFIG_HOME/.bashrc"
            "$XDG_CONFIG_HOME/bash_profile"
            "$XDG_CONFIG_HOME/bashrc"
        )
    fi

    set_manually=true
    for bash_config in "${bash_configs[@]}"; do
        tilde_bash_config=$(tildify "$bash_config")

        if [[ -w $bash_config ]]; then
            {
                echo -e '\n# portal'

                for command in "${commands[@]}"; do
                    echo "$command"
                done
            } >>"$bash_config"

            info "Added \"$tilde_bin_dir\" to \$PATH in \"$tilde_bash_config\""

            refresh_command="source $bash_config"
            set_manually=false
            break
        fi
    done

    if [[ $set_manually = true ]]; then
        echo "Manually add the directory to $tilde_bash_config (or similar):"

        for command in "${commands[@]}"; do
            info_bold "  $command"
        done
    fi
    ;;
*)
    echo 'Manually add the directory to ~/.bashrc (or similar):'
    info_bold "  export $install_env=$quoted_install_dir"
    info_bold "  export PATH=\"$bin_env:\$PATH\""
    ;;
esac

echo
info "To get started, run:"
echo

if [[ $refresh_command ]]; then
    info_bold "  $refresh_command"
fi

info_bold "  portal --help"
