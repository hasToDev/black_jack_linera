#!/bin/bash
# shellcheck disable=SC2181

# ----------------------------------------------------------
# Transfer Initial Amount from Wallet 0 to Wallet 1
# ----------------------------------------------------------

DEFAULT_CHAIN_WALLET_0="e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65"
DEFAULT_CHAIN_WALLET_1="1db1936dad0717597a7743a8353c9c0191c14c3a129b258e9743aec2b4f05d03"
RESET_PASSWORD="L1vzziYAJAiQD6qMQZRFDkf3rADj5g"

linera -w0 transfer --from $DEFAULT_CHAIN_WALLET_0 --to $DEFAULT_CHAIN_WALLET_1 100000000
linera -w0 sync && linera -w0 query-balance
linera -w1 sync && linera -w1 query-balance

# ----------------------------------------------------------
# Open Chain from DEFAULT WALLET 1
# ----------------------------------------------------------

open_chain_from_default_wallet_one() {
  linera -w1 open-chain --from $DEFAULT_CHAIN_WALLET_1 --initial-balance 5000
  if [ $? -ne 0 ]; then
      echo "Open Chain from DEFAULT WALLET 1 failed. Exiting..."
      exit 1
  fi
}

OPEN_ROOM_STATUS_CHAIN=$(open_chain_from_default_wallet_one)
mapfile -t StringArray <<< "$OPEN_ROOM_STATUS_CHAIN"
ROOM_STATUS_CHAIN_ID=${StringArray[1]}

OPEN_ANALYTICS_CHAIN=$(open_chain_from_default_wallet_one)
mapfile -t StringArray <<< "$OPEN_ANALYTICS_CHAIN"
ANALYTICS_CHAIN_ID=${StringArray[1]}

OPEN_PLAYER_STATUS_CHAIN=$(open_chain_from_default_wallet_one)
mapfile -t StringArray <<< "$OPEN_PLAYER_STATUS_CHAIN"
PLAYER_STATUS_CHAIN_ID=${StringArray[1]}

linera -w0 sync && linera -w0 query-balance
linera -w1 sync && linera -w1 query-balance

# ----------------------------------------------------------
# Show Wallet
# ----------------------------------------------------------
echo ""
linera -w0 wallet show
echo ""
linera -w1 wallet show
echo ""

# ----------------------------------------------------------
# Deploy BlackJack App
# ----------------------------------------------------------
deploy_black_jack_app() {
  linera -w1 --wait-for-outgoing-messages project publish-and-create black_jack_chain \
  --json-parameters "{
  \"leaderboard_chain_id\": \"$DEFAULT_CHAIN_WALLET_1\",
  \"leaderboard_pass\": \"$RESET_PASSWORD\",
  \"room_status_chain_id\": \"$ROOM_STATUS_CHAIN_ID\",
  \"analytics_chain_id\": \"$ANALYTICS_CHAIN_ID\",
  \"player_status_chain_id\": \"$PLAYER_STATUS_CHAIN_ID\"
  }"
  if [ $? -ne 0 ]; then
      echo "publish-and-create BlackJack app failed. Exiting..."
      exit 1
  fi
}

BLACK_JACK_APP_ID=$(deploy_black_jack_app)

# ----------------------------------------------------------
# Request BlackJack App to Room Status and Analytics Chain
# ----------------------------------------------------------

linera -w1 request-application --target-chain-id $DEFAULT_CHAIN_WALLET_1 --requester-chain-id "$ROOM_STATUS_CHAIN_ID" "$BLACK_JACK_APP_ID"
  if [ $? -ne 0 ]; then
      echo "request-application for Room Status Chain failed. Exiting..."
      exit 1
  fi

linera -w1 request-application --target-chain-id $DEFAULT_CHAIN_WALLET_1 --requester-chain-id "$ANALYTICS_CHAIN_ID" "$BLACK_JACK_APP_ID"
  if [ $? -ne 0 ]; then
      echo "request-application for Analytics Chain failed. Exiting..."
      exit 1
  fi

linera -w1 request-application --target-chain-id $DEFAULT_CHAIN_WALLET_1 --requester-chain-id "$PLAYER_STATUS_CHAIN_ID" "$BLACK_JACK_APP_ID"
  if [ $? -ne 0 ]; then
      echo "request-application for Player Status Chain failed. Exiting..."
      exit 1
  fi

# ----------------------------------------------------------
# Create Multi Owner Chain on Wallet 1
# ----------------------------------------------------------

MULTI_OWNER_WALLET_NUMBER=10

for _ in $(seq 1 $MULTI_OWNER_WALLET_NUMBER)
do
    PUB_KEY_0=$(linera -w0 keygen)
    PUB_KEY_1=$(linera -w1 keygen)

    open_multi_chain() {
      linera -w1 open-multi-owner-chain \
      --from $DEFAULT_CHAIN_WALLET_1 \
      --owner-public-keys "$PUB_KEY_0" "$PUB_KEY_1" \
      --multi-leader-rounds 2 \
      --initial-balance 1000
      if [ $? -ne 0 ]; then
          echo "open-multi-owner-chain failed. Exiting..."
          exit 1
      fi
    }

    OPEN_MULTI_CHAIN_RESULT=$(open_multi_chain)

    mapfile -t StringArray <<< "$OPEN_MULTI_CHAIN_RESULT"

    MESSAGE_ID=${StringArray[0]}
    NEW_CHAIN_ID=${StringArray[1]}

    # Assign Public Key to New Multi Chain
    linera -w0 assign --key "$PUB_KEY_0" --message-id "$MESSAGE_ID"
    if [ $? -ne 0 ]; then
        echo "chain assign failed. Exiting..."
        exit 1
    fi

    linera -w1 assign --key "$PUB_KEY_1" --message-id "$MESSAGE_ID"
    if [ $? -ne 0 ]; then
        echo "chain assign failed. Exiting..."
        exit 1
    fi

    # OPTIONAL: for delay on VPS
    sleep 1

    # Request BlackJack App to New Multi Chain
    linera -w1 request-application --target-chain-id $DEFAULT_CHAIN_WALLET_1 --requester-chain-id "$NEW_CHAIN_ID" "$BLACK_JACK_APP_ID"
    if [ $? -ne 0 ]; then
         echo "request-application failed. Exiting..."
         exit 1
    fi

    # OPTIONAL: for delay on VPS
    sleep 1

    # Change chain Permission
    linera -w1 change-application-permissions \
    --chain-id "$NEW_CHAIN_ID" \
    --execute-operations "$BLACK_JACK_APP_ID"

done

# ---------------------------------------------------------------------------------
# Change Wallet 1 Default Chain, Room Status Chain, and Analytics Chain Permission
# only allowing operation from BlackJack app
# ---------------------------------------------------------------------------------

linera -w1 change-application-permissions \
--chain-id "$ROOM_STATUS_CHAIN_ID" \
--execute-operations "$BLACK_JACK_APP_ID"

linera -w1 change-application-permissions \
--chain-id "$ANALYTICS_CHAIN_ID" \
--execute-operations "$BLACK_JACK_APP_ID"

linera -w1 change-application-permissions \
--chain-id "$PLAYER_STATUS_CHAIN_ID" \
--execute-operations "$BLACK_JACK_APP_ID"

linera -w1 change-application-permissions \
--chain-id $DEFAULT_CHAIN_WALLET_1 \
--execute-operations "$BLACK_JACK_APP_ID"

# ----------------------------------------------------------
# Sync Query Balance
# ----------------------------------------------------------
linera -w0 sync && linera -w0 query-balance
linera -w1 sync && linera -w1 query-balance

# ----------------------------------------------------------
# Show Wallet
# ----------------------------------------------------------
echo ""
linera -w0 wallet show
echo ""
linera -w1 wallet show
echo ""

# ------------------------------------------------------------
# Show BlackJack App ID, Room Status, Analytics, and Player Status Chain ID
# ------------------------------------------------------------
echo ""
echo "BLACKJACK APP ID:"
echo "$BLACK_JACK_APP_ID"
echo ""
echo "LEADERBOARD CHAIN:"
echo "$DEFAULT_CHAIN_WALLET_1"
echo ""
echo "ROOM STATUS CHAIN:"
echo "$ROOM_STATUS_CHAIN_ID"
echo ""
echo "ANALYTICS CHAIN:"
echo "$ANALYTICS_CHAIN_ID"
echo ""
echo "PLAYER STATUS CHAIN:"
echo "$PLAYER_STATUS_CHAIN_ID"
echo ""