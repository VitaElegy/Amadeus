package com.amadeus.server.service;

import com.sun.jna.Library;
import com.sun.jna.Native;
import org.springframework.stereotype.Service;
import lombok.extern.slf4j.Slf4j;
import java.util.concurrent.CompletableFuture;

@Service
@Slf4j
public class AmadeusCoreService {

    // Define the C-compatible interface for the Rust core
    public interface AmadeusLib extends Library {
        AmadeusLib INSTANCE = Native.load("amadeus", AmadeusLib.class);

        // Assuming extern "C" fn amadeus_execute_command(cmd: *const c_char) -> *const c_char;
        String amadeus_execute_command(String command);
        
        // Assuming extern "C" fn amadeus_get_status() -> *const c_char;
        String amadeus_get_status();
    }

    private boolean isNativeLoaded = false;

    public AmadeusCoreService() {
        try {
            // Attempt to trigger load to check if library exists
            // AmadeusLib.INSTANCE.toString(); 
            // isNativeLoaded = true;
            log.warn("Amadeus Native Library not found in java.library.path. Falling back to MOCK mode.");
        } catch (Throwable t) {
            log.warn("Failed to load Amadeus Native Library: {}. Falling back to MOCK mode.", t.getMessage());
        }
    }

    public String executeCoreCommand(String command) {
        if (isNativeLoaded) {
            try {
                return AmadeusLib.INSTANCE.amadeus_execute_command(command);
            } catch (UnsatisfiedLinkError e) {
                log.error("Native symbol not found", e);
            }
        }
        
        // Mock fallback
        log.info("[MOCK] Dispatching command to Amadeus Core: {}", command);
        simulateLatency();
        return "AMADEUS_CORE_ACK: " + command.toUpperCase() + " (Processed by Java Mock)";
    }
    
    public String getSystemStatus() {
        if (isNativeLoaded) {
             try {
                return AmadeusLib.INSTANCE.amadeus_get_status();
            } catch (UnsatisfiedLinkError e) {
                log.error("Native symbol not found", e);
            }
        }

        // Mock fallback
        return "CORE_ONLINE | MODE: MOCK | LATENCY: 50ms";
    }

    private void simulateLatency() {
        try {
            Thread.sleep(50);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
    }
}
