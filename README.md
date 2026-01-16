# Smart Patient Room Monitor (IoT + Rust + FHIR)

A multi-sensor IoT system for longitudinal activity tracking, environmental safety, and clinical decision support in healthcare facilities.

---

## 📌 Project Overview
This project bridges the gap between simple emergency alarms and comprehensive patient monitoring. By leveraging a hybrid IoT architecture, it combines real-time sensor data with clinical logic to support physiotherapy, elderly care, and mental health monitoring.

The system captures Motion, Sound, and Temperature data, processes it via a high-performance Rust backend, standardizes it into HL7 FHIR R4 resources, and visualizes it on a real-time "Nurse Station" dashboard.

### 🏥 Clinical Use Cases
* Physiotherapy: Validates patient mobility targets via "Average Physical Activity" scores.
* Elderly Care: Monitors "longest still periods" for pressure ulcer prevention and detects wandering.
* Patient Safety: Detects potential falls using a multi-factor algorithm (Motion + Peak Audio Amplitude).
* Mental Health: Tracks circadian rhythm disruptions (e.g., reversed sleep-wake cycles).

🏗️ Technical Architecture

## 1. Hardware Layer (Perception)
* Microcontroller: Arduino Uno R3 acting as the sensor hub.
* Sensors:
    * PIR Motion: For presence and activity intensity.
    * Sound (KY-038): Implements interrupt-based 1000Hz sampling to capture transient impact sounds (solving standard polling limitations).
    * DHT11: For ambient room temperature monitoring.

### 2. Backend Layer (Rust & Actix)
* Framework: Built with Rust and Actix-web for memory safety and high concurrency.
* Concurrency Model: Uses a dedicated background thread for serial ingestion and an actor-based model for WebSocket broadcasting.
* Data Processing:
    * Parses raw CSV streams in real-time.
    * Applies logic for Fall Detection (Simultaneous High Motion + Loud Sound) and Inactivity Alerts.
* Interoperability: Transforms all data into FHIR R4 Observation resources using LOINC (8310-5, 89020-2) and SNOMED CT (52821000) codes.
* Storage: PostgreSQL database with connection pooling for persistent history.

### 3. Frontend Layer (Visualization)
* Stack: Vanilla JavaScript & D3.js (v7).
* Features:
    * Real-time Streaming: Sub-100ms latency via WebSockets.
    * Longitudinal Charts: Visualizes "Rest" vs. "Active" periods over 24h timelines.
    * Audio Alerts: Uses the Web Audio API for distinct, professional warning tones to mitigate alarm fatigue.

## 🛠️ Setup & Deployment

### ☁️ GitHub Codespaces 
This project is fully containerized and includes a **Mock Mode**, allowing you to test the full software stack (Backend, Database, Frontend) without physical sensor hardware.

1.  **Launch Codespace:** Click the green **Code** button > **Codespaces** > **Create codespace on main**.
2.  **Run via Docker:**
    The repository includes a `docker-compose.yml` that orchestrates the Rust backend and PostgreSQL database.
    ```bash
    docker-compose up --build
    ```
3.  **Verify Mock Data:**
    * Since Codespaces cannot access local USB ports, the system automatically detects the environment and switches to `MOCK_MODE`[cite: 69].
    * You will see simulated sensor data (randomized temperature, motion, and sound spikes) streaming to the dashboard immediately.
4.  **Access Dashboard:**
    * Click the "Ports" tab in VS Code.
    * Open the forwarded address for **Port 8080** (or 8000, depending on your config) to view the live dashboard.

---
### Running the Frontend
Simply serve the `frontend` directory using any static file server or open `index.html` directly (backend must be running).

---

## 🧪 Testing
* Includes 81 unit tests covering the Serial Parser, FHIR Transformation, and Alert Logic modules[cite: 73].
* Run tests with: `cargo test`

---

Author: Samuel Safari Onyango  
