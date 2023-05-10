
watch_install() {
  while true
  do
    clear
      cargo install --path . --force --offline
    awaitchange src
  done
}

case $1 in
"help")
  echo "supply install or watch-install as first argument" && exit 1;;
"install")
  cargo install --path . --force --offline;;
"watch-install")
  watch_install;;
pattern-N)
  commands;;
*)

  echo "supply install or watch-install as first argument" && exit 0
  ;;
esac
