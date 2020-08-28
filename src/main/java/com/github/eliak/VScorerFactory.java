package com.github.eliak;

import org.apache.lucene.index.BinaryDocValues;

@FunctionalInterface
public interface VScorerFactory {
    VScorer create(VWeight weight, BinaryDocValues docValues, int docBase);
}
