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
)

// Generic index-based memory storage
type Container[Context any] struct {
	tunnels map[int32]Context
}

func New[Context any]() Container[Context] {
	return Container[Context]{
		tunnels: make(map[int32]Context),
	}
}

func (tc *Container[Context]) Insert(context Context) (int32, error) {
	var i int32
	for i = 0; i < math.MaxInt32; i++ {
		if _, exists := tc.tunnels[i]; !exists {
			break
		}
	}

	if i == math.MaxInt32 {
		return 0, errors.New("container is full")
	}

	tc.tunnels[i] = context
	return i, nil
}

func (tc *Container[Context]) Get(handle int32) (*Context, error) {
	context, ok := tc.tunnels[handle]
	if !ok {
		return nil, errors.New("invalid context handle")
	}
	return &context, nil
}

func (tc *Container[Context]) Remove(handle int32) (*Context, error) {
	context, ok := tc.tunnels[handle]
	if !ok {
		return nil, errors.New("invalid context handle")
	}
	delete(tc.tunnels, handle)
	return &context, nil
}

func (tc *Container[Context]) ForEach(callback func(Context)) {
	for _, tunnel := range tc.tunnels {
		callback(tunnel)
	}
}
