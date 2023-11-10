commit-input:
    cd input && [[ -n $(git status -s) ]] && git add . && git commit -m "$(git status --porcelain | sed 's/A /Add/')" && git push
author-to-commit-date:
    git filter-branch --env-filter 'export GIT_COMMITTER_DATE="$GIT_AUTHOR_DATE"'
cherry-pick commit:
    git cherry-pick {{commit}}
    cargo aoc
    just commit-input
    git commit --amend --no-edit -a
samply day part:
    cargo aoc --profile -d {{day}} -p {{part}}
    samply record -r 1000000000 /Users/dnn/.cargo-target/release/aoc-autobuild
