sudo chmod -R 777 . &&
sudo docker compose --profile generate-wasm up --build &&
sudo chmod -R 777 . &&
sudo docker compose --profile run-app up --build

#sudo chmod -R 777 . &&
#sudo docker compose up wasm-generator &&
#sudo docker compose up code-distributor &&
#sudo docker compose up runtime-code-mobility-demo
