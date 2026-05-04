import { load, Type } from "protobufjs";

const protoMap = new Map<string, Type>();

async function loadProtoFile(filePath: string, lookupKey: string) {
  const cacheKey = `${filePath}:${lookupKey}`;

  if (protoMap.has(cacheKey)) {
    return protoMap.get(cacheKey)!;
  }

  if (filePath.endsWith(".proto")) {
    const root = await load(filePath);
    const messageType = root.lookupType(lookupKey);
    protoMap.set(cacheKey, messageType);

    return messageType;
  } else {
    throw new Error("Invalid file type. Only .proto files are supported.");
  }
}

export async function encodeProtoMessage(
  filePath: string,
  lookupKey: string,
  message: object,
): Promise<Uint8Array> {
  const messageType = await loadProtoFile(filePath, lookupKey);
  const errMsg = messageType.verify(message);
  if (errMsg) {
    throw new Error(errMsg);
  }
  const buffer = messageType.encode(message).finish();
  return buffer;
}

// In protoHelpers.ts - REPLACE your entire decodeProtoMessage function with this:
const protoCache = new Map<string, any>();
export async function decodeProtoMessage<T>(
  filePath: string,
  lookupKey: string,
  buffer: ArrayBufferLike,
): Promise<T> {
  try {
    console.log("📥 decodeProtoMessage START:", {
      filePath,
      lookupKey,
      bufferSize: buffer.byteLength
    });

    const messageType = await loadProtoFile(filePath, lookupKey);

    console.log("📦 Proto loaded:", {
      typeName: messageType?.name || "unknown"
    });

    const uintData = new Uint8Array(buffer);

    console.log("📊 Raw buffer preview:", {
      firstBytes: Array.from(uintData.slice(0, 20))
        .map(b => b.toString(16).padStart(2, "0"))
        .join(" ")
    });

    const message = messageType.decode(uintData) as any;

    console.log("🧬 Decoded raw message:", message);

    const validationError = messageType.verify(message);

    if (validationError) {
      console.warn("⚠️ Proto validation warning:", validationError);
    } else {
      console.log("✅ Proto validation passed");
    }

    const obj = messageType.toObject(message, {
      longs: String,
      enums: String,
      bytes: String,
    }) as unknown as T;

    console.log("📤 Final object output:", obj);
    console.log("FIELD EXISTS:", "isAboveTarget" in message);
    console.log("FIELD EXISTS:", "lessThan40Secs" in message);
    console.log("VALUE:", message.lessThan40Secs);
    return obj;

  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);

    console.error(`❌ decodeProtoMessage FAILED:`, {
      lookupKey,
      filePath,
      error: errorMessage,
      bufferSize: buffer.byteLength
    });

    throw error;
  }
}