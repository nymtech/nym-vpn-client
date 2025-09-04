/* SPDX-License-Identifier: Apache-2.0
 *
 * Copyright (C) 2017-2019 Jason A. Donenfeld <Jason@zx2c4.com>. All Rights Reserved.
 * Copyright (C) 2020 Mullvad VPN AB. All Rights Reserved.
 * Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
 */

package container

import (
	"errors"
	"math"
	"sync"
)

// Generic index-based memory storage
type Container[Context any] struct {
	tunnels map[int32]Context
	lock    sync.RWMutex
}

func New[Context any]() Container[Context] {
	return Container[Context]{
		tunnels: make(map[int32]Context),
		lock:    sync.RWMutex{},
	}
}

func (wself *Container[Context]) Insert(context Context) (int32, error) {
	wself.lock.Lock()
	defer wself.lock.Unlock()

	var i int32
	for i = 0; i < math.MaxInt32; i++ {
		if _, exists := wself.tunnels[i]; !exists {
			break
		}
	}

	if i == math.MaxInt32 {
		return 0, errors.New("container is full")
	}

	wself.tunnels[i] = context
	return i, nil
}

func (wself *Container[Context]) Get(handle int32) (*Context, error) {
	wself.lock.Lock()
	defer wself.lock.Unlock()

	context, ok := wself.tunnels[handle]
	if !ok {
		return nil, errors.New("invalid context handle")
	}
	return &context, nil
}

func (wself *Container[Context]) Remove(handle int32) (*Context, error) {
	wself.lock.Lock()
	defer wself.lock.Unlock()

	context, ok := wself.tunnels[handle]
	if !ok {
		return nil, errors.New("invalid context handle")
	}
	delete(wself.tunnels, handle)
	return &context, nil
}

func (wself *Container[Context]) ForEach(callback func(Context)) {
	// Make a shallow copy of the tunnels map to avoid locking during iteration
	wself.lock.RLock()
	tunnels := wself.tunnels
	wself.lock.Unlock()

	for _, tunnel := range tunnels {
		callback(tunnel)
	}
}
