package com.github.eliak;

import org.apache.lucene.util.BytesRef;

import java.nio.ByteBuffer;
import java.util.Random;

public class BenchUtils {
    public final static int VECTOR_SIZE = 512;

    public static float[] generateArray() {
        return generateArray(VECTOR_SIZE, false);
    }

    public static float[] generateArray(int size) {
        return generateArray(size, false);
    }

    public static float[] generateArray(boolean needDotProduct) {
        return generateArray(VECTOR_SIZE, needDotProduct);
    }

    public static float[] generateArray(int size, boolean needDotProduct) {
        final float[] floats = new float[size + (needDotProduct ? 1 : 0)];
        final Random random = new Random();
        double dotProduct = 0f;
        for (int i = 0; i < size; i++) {
            final float value = random.nextFloat();
            floats[i] = value;
            dotProduct += value * value;
        }
        if(needDotProduct) {
            floats[size] = (float) Math.sqrt(dotProduct);
        }
        return floats;
    }

    public static BytesRef toBytesRef(float[] floats, boolean needDotProduct) {
        final byte[] bytes = new byte[(floats.length + (needDotProduct ? 1 : 0)) * Float.BYTES];
        final ByteBuffer buffer = ByteBuffer.wrap(bytes);
        double dotProduct = 0f;
        for (final float value : floats) {
            buffer.putFloat(value);
            dotProduct += value * value;
        }
        if(needDotProduct) {
            buffer.putFloat((float) Math.sqrt(dotProduct));
        }
        return new BytesRef(bytes);
    }

    public static BytesRef generateBytesRef(boolean needDotProduct) {
        return generateBytesRef(VECTOR_SIZE, needDotProduct);
    }

    public static BytesRef generateBytesRef(int size, boolean needDotProduct) {
        final byte[] bytes = new byte[(size + (needDotProduct ? 1 : 0)) * Float.BYTES];
        final ByteBuffer buffer = ByteBuffer.wrap(bytes);
        final Random random = new Random();
        double dotProduct = 0f;
        for (int i = 0; i < size; i++) {
            final float value = random.nextFloat();
            buffer.putFloat(value);
            dotProduct += value * value;
        }
        if(needDotProduct) {
            buffer.putFloat((float) Math.sqrt(dotProduct));
        }
        return new BytesRef(bytes);
    }
}
