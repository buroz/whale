package main

import (
	"fmt"
	"os/exec"
	"strconv"
	"strings"
	"time"
)

func main() {
	// find erc20_listener pid
	// Find the process ID of erc20_listener
	cmd := exec.Command("pgrep", "erc20_listener")
	output, err := cmd.Output()
	if err != nil {
		fmt.Println("erc20_listener çalışmıyor")
		return
	}

	pid := strings.TrimSpace(string(output))

	// get memory usage of erc20_listener
	cmd = exec.Command("ps", "-p", pid, "-o", "rss=")
	output, err = cmd.Output()
	if err != nil {
		fmt.Println("Memory usage alınamadı")
		return
	}

	ticker := time.NewTicker(3 * time.Second)

	for {
		select {
		case <-ticker.C:
			cmd = exec.Command("ps", "-p", pid, "-o", "rss=")
			output, err = cmd.Output()
			if err != nil {
				fmt.Println("Memory usage alınamadı")
				return
			}
			memoryUsage := strings.TrimSpace(string(output))

			memoryUsageKB, err := strconv.ParseFloat(memoryUsage, 64)
			if err != nil {
				fmt.Println("Memory usage alınamadı")
				return
			}

			memoryUsageMB := memoryUsageKB / 1024.0

			fmt.Printf("Memory usage: %.2f MB\n", memoryUsageMB)
		}
	}
}
