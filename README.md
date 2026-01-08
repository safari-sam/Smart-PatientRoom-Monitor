# 🏥 Smart Patient Room Monitor

A real-time patient monitoring system that detects potential falls and patient inactivity using Arduino sensors, with a FHIR-compliant REST API and live web dashboard.

![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-316192?style=flat&logo=postgresql&logoColor=white)
![Docker](https://img.shields.io/badge/Docker-2496ED?style=flat&logo=docker&logoColor=white)
![FHIR](https://img.shields.io/badge/HL7-FHIR%20R4-red?style=flat)

---

## 🚀 Quick Start

### Option 1: GitHub Codespaces (Easiest - No Installation Required!)

1. Click the green **Code** button above
2. Select **Codespaces** tab
3. Click **Create codespace on main**
4. Wait ~2-3 minutes for the environment to build
5. Once ready, run in the terminal:
   ```bash
   docker-compose up --build
   ```
6. Click the **Ports** tab → Click the 🌐 globe icon next to port **8080**
7. **Dashboard opens in your browser!** 🎉

### Option 2: Local Docker

```bash
# Clone the repository
git clone https://github.com/YOUR_USERNAME/smart-patient-monitor.git
cd smart-patient-monitor

# Start with Docker Compose
docker-compose up --build

# Open http://localhost:8080
```

### Option 3: Local Development

```bash
# Prerequisites: Rust 1.70+, PostgreSQL

# Start PostgreSQL
docker run -d --name patient-db \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=patient_monitor \
  -p 5432:5432 postgres:15-alpine

# Run the backend
cd backend
cp ../.env.example .env
cargo run

# Open http://localhost:8080
```

---

## 📊 Features

| Feature | Description |
|---------|-------------|
| **Real-time Monitoring** | Live sensor data via WebSocket |
| **Fall Detection** | Motion + sound correlation triggers alerts |
| **Inactivity Alerts** | Configurable timeout warnings |
| **Activity Reports** | Sleep analysis with D3.js visualizations |
| **FHIR API** | HL7 FHIR R4 compliant data exchange |
| **Audio Alerts** | Sound + voice notifications |
| **Settings UI** | Adjustable thresholds from dashboard |
| **81 Unit Tests** | Comprehensive test coverage |

---

## 🏗 Architecture

```
┌─────────────────┐      Serial       ┌─────────────────────────┐
│  Arduino Uno    │ ────────────────► │     Rust Backend        │
│  ┌───────────┐  │                   │  ┌─────────────────┐    │
│  │PIR Motion │  │                   │  │ Actix-Web       │    │
│  │Sound Sensor│ │                   │  │ ├─ REST API     │    │
│  │DHT11 Temp │  │                   │  │ └─ WebSocket    │    │
│  └───────────┘  │                   │  └─────────────────┘    │
└─────────────────┘                   │           │             │
                                      │  ┌────────▼────────┐    │
                                      │  │   PostgreSQL    │    │
                                      │  └─────────────────┘    │
                                      └───────────┬─────────────┘
                                                  │
                                      ┌───────────▼─────────────┐
                                      │   Web Dashboard (D3.js) │
                                      │  ├─ Real-time Charts    │
                                      │  ├─ Alert System        │
                                      │  └─ Activity Reports    │
                                      └─────────────────────────┘
```

---

## 📡 API Endpoints

### REST API (FHIR-Compliant)

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/health` | GET | Health check |
| `/api/observations` | GET | FHIR Bundle of sensor readings |
| `/api/observations/latest` | GET | Most recent observation |
| `/api/observations/{id}` | GET | Specific observation by ID |
| `/api/summary` | GET | Alert statistics |
| `/api/activity/sleep` | GET | Sleep analysis report |
| `/api/settings` | GET/POST | Monitor configuration |

### WebSocket

Connect to `/ws` for real-time sensor data streaming.

### Example FHIR Response

```json
{
  "resourceType": "Observation",
  "id": "observation-123",
  "status": "final",
  "code": {
    "coding": [{
      "system": "http://loinc.org",
      "code": "85353-1",
      "display": "Vital signs panel"
    }]
  },
  "component": [
    {
      "code": {"coding": [{"code": "8310-5", "display": "Body temperature"}]},
      "valueQuantity": {"value": 23.5, "unit": "Cel"}
    },
    {
      "code": {"coding": [{"code": "52821000", "display": "Motion detected"}]},
      "valueBoolean": true
    }
  ]
}
```

---

## 🧪 Running Tests

```bash
cd tests
cargo test

# With verbose output
cargo test -- --nocapture

# Run specific test module
cargo test alert
cargo test fhir
cargo test api
```

**Test Coverage:** 81 test cases covering alert logic, FHIR compliance, API responses, activity analysis, and database operations.

---

## ⚙️ Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | `0.0.0.0` | Server bind address |
| `PORT` | `8080` | Server port |
| `DATABASE_URL` | `postgres://...` | PostgreSQL connection |
| `MOCK_MODE` | `true` | Simulate sensor data |
| `SOUND_THRESHOLD` | `150` | Fall detection sensitivity |
| `INACTIVITY_SECONDS` | `300` | Inactivity alert timeout |

---

## 📁 Project Structure

```
smart_patient_monitor/
├── backend/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs          # Entry point
│   │   ├── api.rs           # REST endpoints
│   │   ├── db.rs            # Database operations
│   │   ├── fhir.rs          # FHIR data models
│   │   ├── serial.rs        # Arduino communication
│   │   └── websocket.rs     # Real-time streaming
│   └── frontend/
│       ├── index.html
│       ├── styles.css
│       └── app.js
├── tests/                    # Unit & integration tests
├── .devcontainer/            # GitHub Codespaces config
├── Dockerfile
├── docker-compose.yml
└── README.md
```

---

## 👤 Author

**Sammy**  
BSc Health Informatics, 3rd Semester  
Deggendorf Institute of Technology

---

## 📄 License

Educational project - Web Application Development, Winter Semester 2024/2025
