package lib

//import (
//	"bytes"
//	"encoding/binary"
//)
//
//const RegionSize = 12
//
////Region 内存指针
//type Region struct {
//	Offset   uint32
//	Capacity uint32
//	Length   uint32
//}
//
//func NewRegion(memory []byte) Region {
//	var ret Region
//	bytesBuffer := bytes.NewBuffer(memory)
//	_ = binary.Read(bytesBuffer, binary.LittleEndian, &ret.Offset)
//	_ = binary.Read(bytesBuffer, binary.LittleEndian, &ret.Capacity)
//	_ = binary.Read(bytesBuffer, binary.LittleEndian, &ret.Length)
//	return ret
//}
//
//func (region Region) ToBytes() []byte {
//	bytesBuffer := bytes.NewBuffer([]byte{})
//	_ = binary.Write(bytesBuffer, binary.LittleEndian, region.Offset)
//	_ = binary.Write(bytesBuffer, binary.LittleEndian, region.Capacity)
//	_ = binary.Write(bytesBuffer, binary.LittleEndian, region.Length)
//	return bytesBuffer.Bytes()
//}
//
//func (region Region) GetData(memory []byte) []byte {
//	return memory[region.Offset : region.Offset+region.Length]
//}
