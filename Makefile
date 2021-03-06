all: docs/bld/main_flow.svg docs/bld/rfm_packet.svg docs/bld/type_3_packet.svg docs/bld/type_4_packet.svg docs/bld/type_5_packet.svg

docs/bld/main_flow.svg: docs/src/main_flow.diag
	blockdiag -T svg -o docs/bld/main_flow.svg docs/src/main_flow.diag

docs/bld/rfm_packet.svg: docs/src/rfm_packet.diag
	packetdiag -T svg -o docs/bld/rfm_packet.svg docs/src/rfm_packet.diag

docs/bld/type_3_packet.svg: docs/src/type_3_packet.diag
	packetdiag -T svg -o docs/bld/type_3_packet.svg docs/src/type_3_packet.diag

docs/bld/type_4_packet.svg: docs/src/type_4_packet.diag
	packetdiag -T svg -o docs/bld/type_4_packet.svg docs/src/type_4_packet.diag

docs/bld/type_5_packet.svg: docs/src/type_5_packet.diag
	packetdiag -T svg -o docs/bld/type_5_packet.svg docs/src/type_5_packet.diag
