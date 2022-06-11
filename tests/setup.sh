set -a # automatically export all variables
source .env
set +a

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;36m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

REGEX_PRINCIPAL='(?:[a-z0-9]+\-[a-z0-9]+)+'
REGEX_NAT='[0-9_]+ : nat'

trap 'catch "$LINENO" "$?"' ERR
trap 'finish' EXIT
#set -x
#set -e
echo_e() {
  if [ "$(uname)" == "Darwin" ]; then
    echo "$@"
  else
    echo -e "$@"
  fi
}
info() {
  echo_e "${BLUE}[info] ${NC}" "$@"
}
catch() {
  echo_e "${RED}ERROR${NC} at line $1" # "with code $2"
  if [ -n "$EXIT_IF_ERROR" ]; then
    dfx identity use default
    exit 1
  fi
  CURRENT_FAILED=true
  return 0
}

start() {
  if [ -n "$CURRENT_STEP" ]; then
    echo_e "${RED}SYNTAX ERROR:${NC} previous 'start' does not have a matching 'end'"
    return 1
  fi
  CURRENT_STEP=$1
  echo_e "${BLUE}start${NC} $1"
}

TOTAL_TESTS=0
PASSED_TESTS=0
end() {
  if [ -z "$CURRENT_STEP" ]; then
    echo_e "${RED}SYNTAX ERROR:${NC} 'end' used before 'start'"
    return 1
  fi
  if [ -n "$CURRENT_FAILED" ]; then
    echo_e "${RED}failed${NC} $CURRENT_STEP\n"
  else
    PASSED_TESTS=$((PASSED_TESTS + 1))
    echo_e "${GREEN}passed${NC} $CURRENT_STEP\n"
  fi
  TOTAL_TESTS=$((TOTAL_TESTS + 1))
  CURRENT_STEP=
  CURRENT_FAILED=
}

run() {
  if [ -n "$CURRENT_FAILED" ]; then
    return 1
  fi
}

finish() {
  # shellcheck disable=SC2181
  if [ $? -ne 0 ]; then
    return $?
  fi
  if [ $TOTAL_TESTS == $PASSED_TESTS ]; then
    echo_e "${GREEN}$PASSED_TESTS/$TOTAL_TESTS tests passed${NC}\n"
  else
    echo_e "${YELLOW}$PASSED_TESTS/$TOTAL_TESTS tests passed${NC}\n"
  fi
}

assert_eq() {
  if [[ "$1" != "$2" ]]; then
    echo_e "${RED}ASSERT FAIL:${NC} $1 != $2"
    return 1
  fi
}
