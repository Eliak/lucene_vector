package com.github.eliak;

import org.apache.lucene.analysis.standard.StandardAnalyzer;
import org.apache.lucene.document.BinaryDocValuesField;
import org.apache.lucene.document.Document;
import org.apache.lucene.index.DirectoryReader;
import org.apache.lucene.index.IndexReader;
import org.apache.lucene.index.IndexWriter;
import org.apache.lucene.index.IndexWriterConfig;
import org.apache.lucene.search.IndexSearcher;
import org.apache.lucene.store.ByteBuffersDirectory;
import org.openjdk.jmh.annotations.*;

import java.io.IOException;
import java.util.HashMap;
import java.util.Map;

@State(Scope.Benchmark)
public class BenchSearchState {
    public final static String FIELD_NAME = "b";
    public String fieldName = FIELD_NAME;
    public ByteBuffersDirectory dir;
    public IndexWriter writer;
    public IndexReader reader;
    public IndexSearcher searcher;
    private VScoreNative.ScorerFactory scorerFactory;

    @Param({"native", "simpleCache", "default"})
    public String scoreFunction;

    public VQuery query;

    @Setup(Level.Trial)
    public void setUp() throws IOException {
        dir = new ByteBuffersDirectory();
        final StandardAnalyzer analyzer = new StandardAnalyzer();
        final IndexWriterConfig writerConfig = new IndexWriterConfig(analyzer);
        writer = new IndexWriter(dir, writerConfig);

        for (int i = 0; i < 100_000; i++) {
            final Document doc = new Document();
            doc.add(new BinaryDocValuesField(fieldName, BenchUtils.generateBytesRef(true)));
            writer.addDocument(doc);
        }

        writer.commit();

        reader = DirectoryReader.open(writer);
        searcher = new IndexSearcher(reader);
        VScorerFactory scorerFactory;
        switch (scoreFunction) {
            case "simpleCache": {
                final Map<Integer, float[]> cache = new HashMap<>(10);
                scorerFactory = (w, dv, dbase) -> new VScorerSimpleCache(w, dv, dbase, cache);
                break;
            }
            case "native": {
                scorerFactory = new VScoreNative.ScorerFactory();
                break;
            }
            default:
                scorerFactory = VScorerDefault::new;
        }
        query = new VQuery(fieldName, BenchUtils.generateArray(), scorerFactory);
        System.out.println("setUp");
    }

    @TearDown(Level.Trial)
    public void tearDown() throws IOException {
        System.out.println("tearDown");
        if(scorerFactory  != null){
            scorerFactory.close();
        }
        reader.close();
        writer.close();
        dir.close();
    }
}
