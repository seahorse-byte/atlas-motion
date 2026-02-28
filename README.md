# Atlas Motion - Frontend & Project Guide

This is a full-stack real-time logistics tracking application built with Rust and Apache Kafka. 

> **Note:** This README covers running the entire stack. Ensure you run these commands from the correct directories as indicated.

## Prerequisites
Before you begin, ensure you have the following installed:
- **Rust & Cargo** (via rustup)
- **Dioxus CLI**: Required for the frontend. You can install it via:
  ```bash
  cargo install dioxus-cli
  ```
- **Docker & Docker Compose**: Required for running the local Kafka infrastructure.

---

## Getting Started

To run the project, you will need to start three distinct parts: the infrastructure (Kafka), the backend, and the frontend. It is recommended to use **three separate terminal windows or tabs**.

### 1. Start the Kafka Infrastructure
The backend depends on an active Kafka cluster. You must start this first.

Open a terminal and navigate to the project root directory (the parent of `frontend`):
```bash
cd ..
docker-compose up -d
```
*Tip: Once running, you can access the Confluent Control Center in your browser at [http://localhost:9021](http://localhost:9021) to monitor topics and messages.*

### 2. Run the Backend
The backend services are built with Actix and rdkafka.

Open a new terminal, navigate to the `backend` directory from the root, and start the server:
```bash
cd ../backend
cargo run --bin tracker-backend
```

### 3. Run the Frontend
The frontend is built using the Dioxus framework.

Open another new terminal, navigate to the `frontend` directory, and start the Dioxus dev server:
```bash
cd frontend  # (or remain here if already in the frontend directory)
dx serve
```
*The frontend will typically be served at `http://localhost:8080` (or whatever port Dioxus specifies in your terminal).*

---

## Things to Remember

* **Run Order Matters**: Always ensure the Docker containers are fully up and running before starting the `tracker-backend`. If Kafka is unreachable, the backend will fail to connect or crash.
* **Kafka Broker Port**: The broker is exposed to your local machine on port `9094`. Ensure nothing else on your machine is using this port.
* **Graceful Shutdown**: 
  * Stop the frontend and backend by pressing `Ctrl + C` in their respective terminals.
  * Stop the Kafka infrastructure by running `docker-compose down` from the root directory so you don't leave heavy Java containers running in the background.
* **Control Center**: Use the UI at `http://localhost:9021` to manually verify if messages are flowing into the topics correctly while you develop.