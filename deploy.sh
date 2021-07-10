#!/bin/bash




sudo docker stop $(sudo docker ps -a -q) && sudo docker-compose down -v && sudo docker system prune -af --volumes
sudo kill -9 $(sudo lsof -t -i:7365) && sudo kill -9 $(sudo lsof -t -i:8990) && sudo kill -9 $(sudo lsof -t -i:7366) && sudo kill -9 $(sudo lsof -t -i:7367) && chown -R $USER:$USER .
cargo build --bins --release --manifest-path microservices/Cargo.toml


# NOTE - building each microservice separately
# ...
# cargo build --bin auth --release --manifest-path microservices/Cargo.toml
# cargo build --bin report --release --manifest-path microservices/Cargo.toml
# cargo build --bin monitor --release --manifest-path microservices/Cargo.toml
# cargo build --bin device --release --manifest-path microservices/Cargo.toml
# cargo build --bin emulator --release --manifest-path microservices/Cargo.toml
# cargo build --bin publisher --release --manifest-path microservices/Cargo.toml
# cargo build --bin subscriber --release --manifest-path microservices/Cargo.toml
# pm2 start microservices/target/release/auth
# pm2 start microservices/target/release/report
# pm2 start microservices/target/release/monitor
# pm2 start microservices/target/release/device
# pm2 start microservices/target/release/emulator
# pm2 start microservices/target/release/publisher
# pm2 start microservices/target/release/subscriber
# pm2 startup



# TODO - run diesel migration commands for each microservice
diesel setup --migration-dir microservice/auth/migrations/
diesel setup --migration-dir microservice/monitor/migrations/
diesel setup --migration-dir microservice/report/migrations/


# TODO - add docker services for compiled microservices inside microservices/target/release folder
# TODO - also write a script to check that a microservice is up and available for other
#        like checking that the auth microservice is available first then do reporting operations
# ...
sudo docker-compose -f docker-compose.yml build --no-cache && sudo docker-compose up -d --force-recreate --remove-orphans
sudo docker save $(sudo docker images -a -q) | gzip > $HOME/avl.tar.gz && sudo docker load -i $HOME/avl.tar.gz && sudo cp .env $HONE/.env