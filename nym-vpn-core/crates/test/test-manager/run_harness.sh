#!/usr/bin/bash

# export TF_VAR_exoscale_api_key="null"
# export TF_VAR_exoscale_api_secret="null"
# export TF_VAR_data_volume_id="null"
# export TF_VAR_template_id="null"
# export EXOSCALE_SSH_KEY_PATH="/home/jmwample/.ssh/id_local"
# export TEST_HARNESS_MNEMONIC="empty for now"
# 
# ./nym-vpn-core/crates/test/test-manager/run_local.sh


export NYM_TEST_QCOW_IMAGE="/home/jmwample/src/debian-12-nocloud-amd64.qcow2"
# export NYM_TEST_QCOW_IMAGE="/home/jmwample/src/fresh_debian12_cli.qcow2"
export NYM_TEST_VM_CONFIG="debian12"
export TEST_HARNESS_MNEMONIC="invest history common brand trick hunt small barrel assume process wild awesome vivid ensure lumber snow give penalty chronic excess black century hamster file"
export TEST_DIST_DIR="/home/jmwample/svc/Nym/vpn-client/nym-vpn-core/target/release"

# ./test.sh configure
./test.sh run-tests
# ./test.sh run-vm
