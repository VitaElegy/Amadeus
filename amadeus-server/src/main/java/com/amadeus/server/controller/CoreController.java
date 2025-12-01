package com.amadeus.server.controller;

import com.amadeus.server.service.AmadeusCoreService;
import lombok.RequiredArgsConstructor;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.*;

import java.util.Map;

@RestController
@RequestMapping("/api/v1/core")
@RequiredArgsConstructor
@CrossOrigin(origins = "http://localhost:5173") // Restrict to Vue dev server for security
public class CoreController {

    private final AmadeusCoreService coreService;

    @GetMapping("/status")
    public ResponseEntity<Map<String, String>> getStatus() {
        return ResponseEntity.ok(Map.of(
            "status", coreService.getSystemStatus(),
            "backend", "Spring Boot 3.2 + JNA"
        ));
    }

    @PostMapping("/command")
    public ResponseEntity<Map<String, String>> executeCommand(@RequestBody Map<String, String> payload) {
        String cmd = payload.get("command");
        if (cmd == null || cmd.isBlank()) {
            return ResponseEntity.badRequest().body(Map.of("error", "Command cannot be empty."));
        }
        
        String result = coreService.executeCoreCommand(cmd);
        return ResponseEntity.ok(Map.of(
            "original_command", cmd,
            "core_response", result
        ));
    }
}
