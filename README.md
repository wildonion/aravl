# AVL Development Setup

## Requirements

* **Install prerequisite packages:** ```sudo apt install libssl-dev && sudo apt install build-essential && sudo apt install cmake && sudo apt install libpq-dev && cargo install diesel_cli --no-default-features --features postgres && cargo install systemfd cargo-watch```

* **Build cassandra, postgres and adminer docker images(containers):** ```sudo docker stop $(sudo docker ps -a -q) && sudo docker-compose down -v && sudo docker-compose -f docker-compose.yml build && sudo docker-compose up -d --force-recreate --remove-orphans```

## Logging

* **Docker logs:** ```sudo docker images && sudo docker ps && sudo docker-compose -f docker-compose.yml logs -f```

* **Export avl postgres db:** ```sudo docker-compose exec postgres /bin/bash -c "PGPASSWORD=avl pg_dump --username avl --dbname avl" > avl.sql```

* **Import avl postgres db:** ```sudo docker-compose exec -it postgres /bin/bash -c "PGPASSWORD=avl psql --username avl --dbname avl < avl.sql```

* **Accessing avl postgres db:** ```sudo docker-compose exec postgres psql --username=avl --dbname=avl```

* **Accessing avl postgres db bash:** ```sudo docker-compose exec postgres bash```

* **Accessing cassandra db - node1:** ```sudo docker-compose exec cassandra-node1 cqlsh --username=avl --password=avl```

* **Accessing cassandra db - node2:** ```sudo docker-compose exec cassandra-node2 cqlsh --username=avl --password=avl```

* **Accessing cassandra db bash - node1:** ```sudo docker-compose exec cassandra-node1 bash```

* **Accessing cassandra db bash - node2:** ```sudo docker-compose exec cassandra-node2 bash```

* **Accessing kafka broker - node1:** ```sudo docker-compose exec kafka-node1 bash```

* **Accessing kafka broker - node2:** ```sudo docker-compose exec kafka-node2 bash```

* **Accessing kafka broker - node3:** ```sudo docker-compose exec kafka-node3 bash```

## Running Microservices

* **Run _auth_ microservice using one the following commands:** 
    * ```systemfd --no-pid -s http::7366 -- cargo watch -C microservices -x 'run --bin auth'```
    * ```systemfd --no-pid -s http::7366 -- cargo watch -C microservices -x 'run -p auth'```
    * ```cargo watch -C microservices -x 'run --bin auth'```
    * ```cargo watch -C microservices -x 'run -p auth'``` 

* **Run _report_ microservice using one the following commands:** 
    * ```systemfd --no-pid -s http::7367 -- cargo watch -C microservices -x 'run --bin report'```
    * ```systemfd --no-pid -s http::7367 -- cargo watch -C microservices -x 'run -p report'```
    * ```cargo watch -C microservices -x 'run --bin report'```
    * ```cargo watch -C microservices -x 'run -p report'```

* **Run _monitor_ microservice using one the following commands:** 
    * ```systemfd --no-pid -s http::7365 -- cargo watch -C microservices -x 'run --bin monitor'```
    * ```systemfd --no-pid -s http::7365 -- cargo watch -C microservices -x 'run -p monitor'```
    * ```cargo watch -C microservices -x 'run --bin monitor'```
    * ```cargo watch -C microservices -x 'run -p monitor'```

* **Run _device_ microservice using one the following commands:** 
    * ```cargo watch -C microservices -x 'run --bin device'```
    * ```cargo watch -C microservices -x 'run -p device'```

* **Run _device_ emulator microservice:**
    * ```cargo watch -C microservices -x 'run --bin emulator'```

* **Run _streamer_ publisher microservice:**
    * ```cargo watch -C microservices -x 'run --bin publisher'```

* **Run _streamer_ subscriber microservice:**
    * ```cargo watch -C microservices -x 'run --bin subscriber'```

# AVL Production Setup

* **Build & run each microservice:** ```sudo chmod +x deploy.sh && ./deploy.sh```

# AVL Postgres Database Setup

* **Generate _migrations_ folder, create avl postgres db, `diesel.toml` file on first run or run existing migrations into the database:** 

    * ```diesel setup --migration-dir microservices/auth/migrations/```

    * ```diesel setup --migration-dir microservices/monitor/migrations/```

    * ```diesel setup --migration-dir microservices/report/migrations/```

* **Generate SQL files for your table operations:** ```diesel migration generate SQL-OPERATION_TABLE-NAME```

    * **eg - create users table for _auth_ microservice:** ```diesel migration generate create_users --migration-dir microservices/auth/migrations/```

* **Migrate tables into postgres db and generate(update) `schema.rs` file inside _src_ folder:** ```diesel migration run```

    * **eg - migrate all SQL files of operations of _auth_ microservice into the database:** ```diesel migration run --migration-dir microservices/auth/migrations/```

* **Check diesel migrations errors:** ```diesel migration list```

    * **eg - check migrations errors for _auth_ microservice:** ```diesel migration list --migration-dir microservices/auth/migrations/```

# Hints

* Remember to install _rustup_, _pm2_, _docker_ and _docker-compose_
* Run `deploy.sh` script only once in your server
* for _TCP_ and _UDP_ handlers of _device_ microservice we've used _tokio_ streamer which is a multithreaded asynchronous task handler based on _mpsc_ job queue channel protocol
* Instead of using _tokio_ socket with _mpsc_ job queue channel protocol to share incoming _GPS_ tasks and data between threads in our _UI_ apps we've used _kafka_ for heavy long time streaming with load balancing and data repications strategy
* The _emulator_ app is a synchronous multithreaded socket and thread safe task handler based on _mpsc_ job queue channel protocol
* For security reasons you must forward all microservices port to something else using nginx or traefik on your VPS
* Currently there are three cassandra nodes and three kafka brokers(nodes) inside our VPS or cluster(datacenter) built and ran with docker
* _cassandra_ => multiple cluster(datacenter or VPS) <-has-> nodes(multiple instances of cassandra db server) <-has-> partition replicas <-has-> rows
* _kafka_ => multiple cluster(datacenter or VPS) <-has-> nodes(multiple instances of kafka brokers or servers) <-has-> topics <-has-> partition replicas for each topic <-has-> buck of events inside each partition 
* Three replicas in cassandra means there are three copies of each partition(contains rows) in each node(cassandra db server)
* Three replicas in kafka means there are three copies of each topics' partitions(buck of events) in each node(kafka broker)
* _kafka_ partitions are created based on the hash of each event and events with similar hash will be inside a same partition so a topic is divided into one or more partitions
* The default number of partitions for each topic is 10
* Down migration command for each table is: ```diesel migration down```
* Adminer page address is: **http://SERVER-IP:3257**
* _monitor_ microservice API address is: **http://SERVER-IP:7365/avl/api/monitor**
* _device_ microservice API address is: **http://SERVER-IP:7366/avl/api/auth**
* _report_ microservice API address is: **http://SERVER-IP:7367/avl/api/report**
* _device_ microservice socket address is: **SERVER-IP:8990**
* In order to generate the `schema.rs` in _src_ folder the ```diesel migration run``` command must have a successful result
* You can also create sql files(`up.sql` and `down.sql`) for your table in each migrations folder by hand then run the ```diesel setup``` command to migrate them all at once