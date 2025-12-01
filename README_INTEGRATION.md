# Amadeus Enterprise Integration

This project integrates the high-performance Amadeus Rust core with a "mature" enterprise stack: Spring Boot backend and Vue 3 frontend.

## 🚀 Quick Start

The easiest way to run the full stack (if you have Java 17+, Maven, and Node.js 18+ installed):

```bash
./run_all.sh
```

## 🏗 Architecture

### 1. Frontend (`amadeus-web`)
A modern, component-based Single Page Application (SPA) designed with Github's Dark Theme aesthetics.

*   **Framework**: Vue 3 (Composition API) + Vite
*   **State Management**: Pinia (Centralized store for Articles, Todos, Memos)
*   **Routing**: Vue Router (Client-side routing for Dashboard, Knowledge Base, etc.)
*   **UI Library**: TDesign Vue Next + Tailwind CSS
*   **Features**:
    *   **Dashboard**: Real-time system status and command dispatcher.
    *   **Knowledge Base**: Article management with card view.
    *   **Task Master**: Fully functional Todo list with add/toggle/delete actions.
    *   **Memos & Notes**: Integrated note-taking for quick memos and vocabulary.

### 2. Middleware (`amadeus-server`)
A Spring Boot wrapper acting as a bridge between the Web UI and the Rust Core.

*   **Framework**: Spring Boot 3.2
*   **Database**: H2 (In-memory) / JPA
*   **Core Bridge**: JNA (Java Native Access)
    *   Attempts to load `libamadeus.dylib` / `libamadeus.so`.
    *   **Fallback Mechanism**: If the native library is missing, it automatically degrades to a Mock mode, simulating core responses and latency.

### 3. Core (`amadeus`)
The underlying Rust plugin system.

*   **Role**: High-performance message processing, plugin management, and system orchestration.
*   **Integration**: Compiled as a C-compatible shared library (`cdylib`) for JNA consumption.

## 🛠 Manual Setup

### Backend
```bash
cd amadeus-server
mvn spring-boot:run
```
*Port: 8080*

### Frontend
```bash
cd amadeus-web
npm install
npm run dev
```
*Port: 5173*

## 📂 Project Structure

```
.
├── amadeus/                 # Rust Core
├── amadeus-server/          # Spring Boot Middleware
│   └── src/main/java/com/amadeus/server/service/AmadeusCoreService.java  # JNA Bridge
├── amadeus-web/             # Vue 3 Frontend
│   ├── src/
│   │   ├── layouts/         # AppLayout (Sidebar + Content)
│   │   ├── views/           # Page Components (Dashboard, Articles, Todos, Notes)
│   │   ├── stores/          # Pinia State Stores
│   │   ├── router/          # Route Definitions
│   │   └── components/      # Reusable UI Components
│   └── tailwind.config.js   # Github Theme Configuration
└── run_all.sh               # One-click startup script
```
