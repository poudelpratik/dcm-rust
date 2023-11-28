sudo chmod -R 777 . &&
sudo docker compose --profile generate-wasm up --build &&
sudo chmod -R 777 . &&
sudo docker compose --profile run-app up --build
