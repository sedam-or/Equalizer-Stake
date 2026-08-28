package main

import (
	"log"
	"os"
)

func main() {
	port := os.Getenv("APP_PORT")
	if port == "" {
		port = "8080"
	}
	log.Printf("Equalizer-Stake server starting on :%s", port)
}
