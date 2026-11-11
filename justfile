_default:
    @just -l

commit-input:
    cd input && [[ -n $(git status -s) ]] && git add . && git commit -m "$(git status --porcelain | sed 's/A /Add/')" && git push

cherry-pick commit:
    git cherry-pick {{commit}}
    cargo aoc-harness run
    just commit-input
    git commit --amend --no-edit -a

samply day part:
    cargo aoc-harness profile --day {{day}} --part {{part}} --family solver --profiler samply

perf day part:
    cargo aoc-harness profile --day {{day}} --part {{part}} --family solver --profiler perf

author-to-commit-date:
    git filter-branch --env-filter 'export GIT_COMMITTER_DATE="$GIT_AUTHOR_DATE"'

watch-test day:
    cargo watch test -- $(printf "day%02d" {{day}}) -- --no-capture --test-threads=1

wait-get-input day:
    caffeinate -disu sleep-progress -u "11:00:04pm" -p
    cargo aoc-harness input --day {{day}}
    (cd input && git checkout main)
    (cd src/common && git checkout main)
