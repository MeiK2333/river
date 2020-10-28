# -*- coding: utf-8 -*-
# Generated by the protocol buffer compiler.  DO NOT EDIT!
# source: river.proto
"""Generated protocol buffer code."""
from google.protobuf.internal import enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from google.protobuf import reflection as _reflection
from google.protobuf import symbol_database as _symbol_database
# @@protoc_insertion_point(imports)

_sym_db = _symbol_database.Default()




DESCRIPTOR = _descriptor.FileDescriptor(
  name='river.proto',
  package='river',
  syntax='proto3',
  serialized_options=None,
  create_key=_descriptor._internal_create_key,
  serialized_pb=b'\n\x0briver.proto\x12\x05river\"\x1b\n\x0b\x43ompileData\x12\x0c\n\x04\x63ode\x18\x01 \x01(\x0c\"X\n\tJudgeData\x12\x0f\n\x07in_data\x18\x01 \x01(\x0c\x12\x10\n\x08out_data\x18\x02 \x01(\x0c\x12\x12\n\ntime_limit\x18\x03 \x01(\x05\x12\x14\n\x0cmemory_limit\x18\x04 \x01(\x05\"\xb3\x01\n\x0cJudgeRequest\x12!\n\x08language\x18\x01 \x01(\x0e\x32\x0f.river.Language\x12$\n\njudge_type\x18\x02 \x01(\x0e\x32\x10.river.JudgeType\x12*\n\x0c\x63ompile_data\x18\x03 \x01(\x0b\x32\x12.river.CompileDataH\x00\x12&\n\njudge_data\x18\x04 \x01(\x0b\x32\x10.river.JudgeDataH\x00\x42\x06\n\x04\x64\x61ta\"\xbc\x01\n\rJudgeResponse\x12\x11\n\ttime_used\x18\x01 \x01(\x03\x12\x13\n\x0bmemory_used\x18\x02 \x01(\x03\x12$\n\x06result\x18\x03 \x01(\x0e\x32\x12.river.JudgeResultH\x00\x12$\n\x06status\x18\t \x01(\x0e\x32\x12.river.JudgeStatusH\x00\x12\x0e\n\x06stdout\x18\x06 \x01(\t\x12\x0e\n\x06stderr\x18\x07 \x01(\t\x12\x0e\n\x06\x65rrmsg\x18\x08 \x01(\tB\x07\n\x05state*R\n\x08Language\x12\x05\n\x01\x43\x10\x00\x12\x07\n\x03\x43pp\x10\x01\x12\n\n\x06Python\x10\x02\x12\x08\n\x04Rust\x10\x03\x12\x08\n\x04Node\x10\x04\x12\x0e\n\nTypeScript\x10\x05\x12\x06\n\x02Go\x10\x06*\x19\n\tJudgeType\x12\x0c\n\x08Standard\x10\x00*\xc1\x01\n\x0bJudgeResult\x12\x0c\n\x08\x41\x63\x63\x65pted\x10\x00\x12\x0f\n\x0bWrongAnswer\x10\x01\x12\x15\n\x11TimeLimitExceeded\x10\x02\x12\x17\n\x13MemoryLimitExceeded\x10\x03\x12\x10\n\x0cRuntimeError\x10\x04\x12\x17\n\x13OutputLimitExceeded\x10\x05\x12\x10\n\x0c\x43ompileError\x10\x06\x12\x15\n\x11PresentationError\x10\x07\x12\x0f\n\x0bSystemError\x10\x08*2\n\x0bJudgeStatus\x12\x0b\n\x07Pending\x10\x00\x12\x0b\n\x07Running\x10\x01\x12\t\n\x05\x45nded\x10\x02\x32\x41\n\x05River\x12\x38\n\x05Judge\x12\x13.river.JudgeRequest\x1a\x14.river.JudgeResponse\"\x00(\x01\x30\x01\x62\x06proto3'
)

_LANGUAGE = _descriptor.EnumDescriptor(
  name='Language',
  full_name='river.Language',
  filename=None,
  file=DESCRIPTOR,
  create_key=_descriptor._internal_create_key,
  values=[
    _descriptor.EnumValueDescriptor(
      name='C', index=0, number=0,
      serialized_options=None,
      type=None,
      create_key=_descriptor._internal_create_key),
    _descriptor.EnumValueDescriptor(
      name='Cpp', index=1, number=1,
      serialized_options=None,
      type=None,
      create_key=_descriptor._internal_create_key),
    _descriptor.EnumValueDescriptor(
      name='Python', index=2, number=2,
      serialized_options=None,
      type=None,
      create_key=_descriptor._internal_create_key),
    _descriptor.EnumValueDescriptor(
      name='Rust', index=3, number=3,
      serialized_options=None,
      type=None,
      create_key=_descriptor._internal_create_key),
    _descriptor.EnumValueDescriptor(
      name='Node', index=4, number=4,
      serialized_options=None,
      type=None,
      create_key=_descriptor._internal_create_key),
    _descriptor.EnumValueDescriptor(
      name='TypeScript', index=5, number=5,
      serialized_options=None,
      type=None,
      create_key=_descriptor._internal_create_key),
    _descriptor.EnumValueDescriptor(
      name='Go', index=6, number=6,
      serialized_options=None,
      type=None,
      create_key=_descriptor._internal_create_key),
  ],
  containing_type=None,
  serialized_options=None,
  serialized_start=514,
  serialized_end=596,
)
_sym_db.RegisterEnumDescriptor(_LANGUAGE)

Language = enum_type_wrapper.EnumTypeWrapper(_LANGUAGE)
_JUDGETYPE = _descriptor.EnumDescriptor(
  name='JudgeType',
  full_name='river.JudgeType',
  filename=None,
  file=DESCRIPTOR,
  create_key=_descriptor._internal_create_key,
  values=[
    _descriptor.EnumValueDescriptor(
      name='Standard', index=0, number=0,
      serialized_options=None,
      type=None,
      create_key=_descriptor._internal_create_key),
  ],
  containing_type=None,
  serialized_options=None,
  serialized_start=598,
  serialized_end=623,
)
_sym_db.RegisterEnumDescriptor(_JUDGETYPE)

JudgeType = enum_type_wrapper.EnumTypeWrapper(_JUDGETYPE)
_JUDGERESULT = _descriptor.EnumDescriptor(
  name='JudgeResult',
  full_name='river.JudgeResult',
  filename=None,
  file=DESCRIPTOR,
  create_key=_descriptor._internal_create_key,
  values=[
    _descriptor.EnumValueDescriptor(
      name='Accepted', index=0, number=0,
      serialized_options=None,
      type=None,
      create_key=_descriptor._internal_create_key),
    _descriptor.EnumValueDescriptor(
      name='WrongAnswer', index=1, number=1,
      serialized_options=None,
      type=None,
      create_key=_descriptor._internal_create_key),
    _descriptor.EnumValueDescriptor(
      name='TimeLimitExceeded', index=2, number=2,
      serialized_options=None,
      type=None,
      create_key=_descriptor._internal_create_key),
    _descriptor.EnumValueDescriptor(
      name='MemoryLimitExceeded', index=3, number=3,
      serialized_options=None,
      type=None,
      create_key=_descriptor._internal_create_key),
    _descriptor.EnumValueDescriptor(
      name='RuntimeError', index=4, number=4,
      serialized_options=None,
      type=None,
      create_key=_descriptor._internal_create_key),
    _descriptor.EnumValueDescriptor(
      name='OutputLimitExceeded', index=5, number=5,
      serialized_options=None,
      type=None,
      create_key=_descriptor._internal_create_key),
    _descriptor.EnumValueDescriptor(
      name='CompileError', index=6, number=6,
      serialized_options=None,
      type=None,
      create_key=_descriptor._internal_create_key),
    _descriptor.EnumValueDescriptor(
      name='PresentationError', index=7, number=7,
      serialized_options=None,
      type=None,
      create_key=_descriptor._internal_create_key),
    _descriptor.EnumValueDescriptor(
      name='SystemError', index=8, number=8,
      serialized_options=None,
      type=None,
      create_key=_descriptor._internal_create_key),
  ],
  containing_type=None,
  serialized_options=None,
  serialized_start=626,
  serialized_end=819,
)
_sym_db.RegisterEnumDescriptor(_JUDGERESULT)

JudgeResult = enum_type_wrapper.EnumTypeWrapper(_JUDGERESULT)
_JUDGESTATUS = _descriptor.EnumDescriptor(
  name='JudgeStatus',
  full_name='river.JudgeStatus',
  filename=None,
  file=DESCRIPTOR,
  create_key=_descriptor._internal_create_key,
  values=[
    _descriptor.EnumValueDescriptor(
      name='Pending', index=0, number=0,
      serialized_options=None,
      type=None,
      create_key=_descriptor._internal_create_key),
    _descriptor.EnumValueDescriptor(
      name='Running', index=1, number=1,
      serialized_options=None,
      type=None,
      create_key=_descriptor._internal_create_key),
    _descriptor.EnumValueDescriptor(
      name='Ended', index=2, number=2,
      serialized_options=None,
      type=None,
      create_key=_descriptor._internal_create_key),
  ],
  containing_type=None,
  serialized_options=None,
  serialized_start=821,
  serialized_end=871,
)
_sym_db.RegisterEnumDescriptor(_JUDGESTATUS)

JudgeStatus = enum_type_wrapper.EnumTypeWrapper(_JUDGESTATUS)
C = 0
Cpp = 1
Python = 2
Rust = 3
Node = 4
TypeScript = 5
Go = 6
Standard = 0
Accepted = 0
WrongAnswer = 1
TimeLimitExceeded = 2
MemoryLimitExceeded = 3
RuntimeError = 4
OutputLimitExceeded = 5
CompileError = 6
PresentationError = 7
SystemError = 8
Pending = 0
Running = 1
Ended = 2



_COMPILEDATA = _descriptor.Descriptor(
  name='CompileData',
  full_name='river.CompileData',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  create_key=_descriptor._internal_create_key,
  fields=[
    _descriptor.FieldDescriptor(
      name='code', full_name='river.CompileData.code', index=0,
      number=1, type=12, cpp_type=9, label=1,
      has_default_value=False, default_value=b"",
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
  ],
  serialized_start=22,
  serialized_end=49,
)


_JUDGEDATA = _descriptor.Descriptor(
  name='JudgeData',
  full_name='river.JudgeData',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  create_key=_descriptor._internal_create_key,
  fields=[
    _descriptor.FieldDescriptor(
      name='in_data', full_name='river.JudgeData.in_data', index=0,
      number=1, type=12, cpp_type=9, label=1,
      has_default_value=False, default_value=b"",
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
    _descriptor.FieldDescriptor(
      name='out_data', full_name='river.JudgeData.out_data', index=1,
      number=2, type=12, cpp_type=9, label=1,
      has_default_value=False, default_value=b"",
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
    _descriptor.FieldDescriptor(
      name='time_limit', full_name='river.JudgeData.time_limit', index=2,
      number=3, type=5, cpp_type=1, label=1,
      has_default_value=False, default_value=0,
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
    _descriptor.FieldDescriptor(
      name='memory_limit', full_name='river.JudgeData.memory_limit', index=3,
      number=4, type=5, cpp_type=1, label=1,
      has_default_value=False, default_value=0,
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
  ],
  serialized_start=51,
  serialized_end=139,
)


_JUDGEREQUEST = _descriptor.Descriptor(
  name='JudgeRequest',
  full_name='river.JudgeRequest',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  create_key=_descriptor._internal_create_key,
  fields=[
    _descriptor.FieldDescriptor(
      name='language', full_name='river.JudgeRequest.language', index=0,
      number=1, type=14, cpp_type=8, label=1,
      has_default_value=False, default_value=0,
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
    _descriptor.FieldDescriptor(
      name='judge_type', full_name='river.JudgeRequest.judge_type', index=1,
      number=2, type=14, cpp_type=8, label=1,
      has_default_value=False, default_value=0,
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
    _descriptor.FieldDescriptor(
      name='compile_data', full_name='river.JudgeRequest.compile_data', index=2,
      number=3, type=11, cpp_type=10, label=1,
      has_default_value=False, default_value=None,
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
    _descriptor.FieldDescriptor(
      name='judge_data', full_name='river.JudgeRequest.judge_data', index=3,
      number=4, type=11, cpp_type=10, label=1,
      has_default_value=False, default_value=None,
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
    _descriptor.OneofDescriptor(
      name='data', full_name='river.JudgeRequest.data',
      index=0, containing_type=None,
      create_key=_descriptor._internal_create_key,
    fields=[]),
  ],
  serialized_start=142,
  serialized_end=321,
)


_JUDGERESPONSE = _descriptor.Descriptor(
  name='JudgeResponse',
  full_name='river.JudgeResponse',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  create_key=_descriptor._internal_create_key,
  fields=[
    _descriptor.FieldDescriptor(
      name='time_used', full_name='river.JudgeResponse.time_used', index=0,
      number=1, type=3, cpp_type=2, label=1,
      has_default_value=False, default_value=0,
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
    _descriptor.FieldDescriptor(
      name='memory_used', full_name='river.JudgeResponse.memory_used', index=1,
      number=2, type=3, cpp_type=2, label=1,
      has_default_value=False, default_value=0,
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
    _descriptor.FieldDescriptor(
      name='result', full_name='river.JudgeResponse.result', index=2,
      number=3, type=14, cpp_type=8, label=1,
      has_default_value=False, default_value=0,
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
    _descriptor.FieldDescriptor(
      name='status', full_name='river.JudgeResponse.status', index=3,
      number=9, type=14, cpp_type=8, label=1,
      has_default_value=False, default_value=0,
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
    _descriptor.FieldDescriptor(
      name='stdout', full_name='river.JudgeResponse.stdout', index=4,
      number=6, type=9, cpp_type=9, label=1,
      has_default_value=False, default_value=b"".decode('utf-8'),
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
    _descriptor.FieldDescriptor(
      name='stderr', full_name='river.JudgeResponse.stderr', index=5,
      number=7, type=9, cpp_type=9, label=1,
      has_default_value=False, default_value=b"".decode('utf-8'),
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
    _descriptor.FieldDescriptor(
      name='errmsg', full_name='river.JudgeResponse.errmsg', index=6,
      number=8, type=9, cpp_type=9, label=1,
      has_default_value=False, default_value=b"".decode('utf-8'),
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
    _descriptor.OneofDescriptor(
      name='state', full_name='river.JudgeResponse.state',
      index=0, containing_type=None,
      create_key=_descriptor._internal_create_key,
    fields=[]),
  ],
  serialized_start=324,
  serialized_end=512,
)

_JUDGEREQUEST.fields_by_name['language'].enum_type = _LANGUAGE
_JUDGEREQUEST.fields_by_name['judge_type'].enum_type = _JUDGETYPE
_JUDGEREQUEST.fields_by_name['compile_data'].message_type = _COMPILEDATA
_JUDGEREQUEST.fields_by_name['judge_data'].message_type = _JUDGEDATA
_JUDGEREQUEST.oneofs_by_name['data'].fields.append(
  _JUDGEREQUEST.fields_by_name['compile_data'])
_JUDGEREQUEST.fields_by_name['compile_data'].containing_oneof = _JUDGEREQUEST.oneofs_by_name['data']
_JUDGEREQUEST.oneofs_by_name['data'].fields.append(
  _JUDGEREQUEST.fields_by_name['judge_data'])
_JUDGEREQUEST.fields_by_name['judge_data'].containing_oneof = _JUDGEREQUEST.oneofs_by_name['data']
_JUDGERESPONSE.fields_by_name['result'].enum_type = _JUDGERESULT
_JUDGERESPONSE.fields_by_name['status'].enum_type = _JUDGESTATUS
_JUDGERESPONSE.oneofs_by_name['state'].fields.append(
  _JUDGERESPONSE.fields_by_name['result'])
_JUDGERESPONSE.fields_by_name['result'].containing_oneof = _JUDGERESPONSE.oneofs_by_name['state']
_JUDGERESPONSE.oneofs_by_name['state'].fields.append(
  _JUDGERESPONSE.fields_by_name['status'])
_JUDGERESPONSE.fields_by_name['status'].containing_oneof = _JUDGERESPONSE.oneofs_by_name['state']
DESCRIPTOR.message_types_by_name['CompileData'] = _COMPILEDATA
DESCRIPTOR.message_types_by_name['JudgeData'] = _JUDGEDATA
DESCRIPTOR.message_types_by_name['JudgeRequest'] = _JUDGEREQUEST
DESCRIPTOR.message_types_by_name['JudgeResponse'] = _JUDGERESPONSE
DESCRIPTOR.enum_types_by_name['Language'] = _LANGUAGE
DESCRIPTOR.enum_types_by_name['JudgeType'] = _JUDGETYPE
DESCRIPTOR.enum_types_by_name['JudgeResult'] = _JUDGERESULT
DESCRIPTOR.enum_types_by_name['JudgeStatus'] = _JUDGESTATUS
_sym_db.RegisterFileDescriptor(DESCRIPTOR)

CompileData = _reflection.GeneratedProtocolMessageType('CompileData', (_message.Message,), {
  'DESCRIPTOR' : _COMPILEDATA,
  '__module__' : 'river_pb2'
  # @@protoc_insertion_point(class_scope:river.CompileData)
  })
_sym_db.RegisterMessage(CompileData)

JudgeData = _reflection.GeneratedProtocolMessageType('JudgeData', (_message.Message,), {
  'DESCRIPTOR' : _JUDGEDATA,
  '__module__' : 'river_pb2'
  # @@protoc_insertion_point(class_scope:river.JudgeData)
  })
_sym_db.RegisterMessage(JudgeData)

JudgeRequest = _reflection.GeneratedProtocolMessageType('JudgeRequest', (_message.Message,), {
  'DESCRIPTOR' : _JUDGEREQUEST,
  '__module__' : 'river_pb2'
  # @@protoc_insertion_point(class_scope:river.JudgeRequest)
  })
_sym_db.RegisterMessage(JudgeRequest)

JudgeResponse = _reflection.GeneratedProtocolMessageType('JudgeResponse', (_message.Message,), {
  'DESCRIPTOR' : _JUDGERESPONSE,
  '__module__' : 'river_pb2'
  # @@protoc_insertion_point(class_scope:river.JudgeResponse)
  })
_sym_db.RegisterMessage(JudgeResponse)



_RIVER = _descriptor.ServiceDescriptor(
  name='River',
  full_name='river.River',
  file=DESCRIPTOR,
  index=0,
  serialized_options=None,
  create_key=_descriptor._internal_create_key,
  serialized_start=873,
  serialized_end=938,
  methods=[
  _descriptor.MethodDescriptor(
    name='Judge',
    full_name='river.River.Judge',
    index=0,
    containing_service=None,
    input_type=_JUDGEREQUEST,
    output_type=_JUDGERESPONSE,
    serialized_options=None,
    create_key=_descriptor._internal_create_key,
  ),
])
_sym_db.RegisterServiceDescriptor(_RIVER)

DESCRIPTOR.services_by_name['River'] = _RIVER

# @@protoc_insertion_point(module_scope)
