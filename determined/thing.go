package main

import (
	"fmt"
	"runtime"
	"sync"
	"time"
)

func main() {
	go blockSleep()
	go blockChan()
	go blockMutex()
	go blockWaitGroup()

	fmt.Println("Booting blocking goroutines...")
	time.Sleep(time.Second * 3)
	capture()

	fmt.Println("\n\n\nBooting notblocking goroutines...")
	go notblockSleep()
	go notblockChan()
	go notblockMutex()
	go notblockWaitGroup()

	time.Sleep(time.Second * 3)
	capture()
}

func blockSleep() {
	time.Sleep(99999999 * time.Second)
}

func blockChan() {
	foo := make(chan struct{})
	foo <- struct{}{}
}

func blockMutex() {
	mtx := new(sync.Mutex)
	mtx.Lock()
	mtx.Lock()
}

func blockWaitGroup() {
	var wg sync.WaitGroup
	wg.Add(1)
	wg.Wait()
}

// line 41

func notblockSleep() {
	for {
		time.Sleep(0 * time.Second)
	}
}

func notblockChan() {
	foo := make(chan struct{})
	for {
		select {
		case foo <- struct{}{}:
		default:
		}
	}
}

func notblockMutex() {
	mtx := new(sync.Mutex)
	for {
		mtx.Lock()
		mtx.Unlock()
	}
}

func notblockWaitGroup() {
	var wg sync.WaitGroup
	for {
		wg.Wait()
	}
}

func capture() {
	trace := make([]byte, 102400)
	count := runtime.Stack(trace, true)
	fmt.Printf("Stack of %d bytes: %s\n", count, trace)
}
