# code repurposed from zoxide/templates/fish.txt

function kn
    set argc (count $argv)

    if test $argc -eq 0
        # no args provided, go to home dir
        
        cd $HOME
    else if begin; test $argc -eq 1; and test $argv[1] = '-'; end
        # only dash provided, go to previous location
        
        cd -
    else
        # otherwise, query _kn
        
        set -l __kn_result (command _kn query $argv)

        and if test -d $__kn_result
            cd $__kn_result
        end
    end
end
