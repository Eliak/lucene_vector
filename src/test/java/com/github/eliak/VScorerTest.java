package com.github.eliak;

import org.apache.lucene.analysis.standard.StandardAnalyzer;
import org.apache.lucene.document.BinaryDocValuesField;
import org.apache.lucene.document.Document;
import org.apache.lucene.index.DirectoryReader;
import org.apache.lucene.index.IndexReader;
import org.apache.lucene.index.IndexWriter;
import org.apache.lucene.index.IndexWriterConfig;
import org.apache.lucene.search.IndexSearcher;
import org.apache.lucene.search.TopDocs;
import org.apache.lucene.store.ByteBuffersDirectory;
import org.testng.annotations.AfterClass;
import org.testng.annotations.BeforeClass;
import org.testng.annotations.Test;

import java.io.IOException;
import java.util.Arrays;
import java.util.HashMap;

import static com.github.eliak.ScoreUtils.*;
import static org.testng.Assert.assertEquals;
import static org.testng.Assert.assertNotNull;

public class VScorerTest {
    ByteBuffersDirectory dir;
    IndexWriter writer;
    float[] floats;

    @BeforeClass
    public void setUp() throws IOException {
        dir = new ByteBuffersDirectory();
        final StandardAnalyzer analyzer = new StandardAnalyzer();
        final IndexWriterConfig writerConfig = new IndexWriterConfig(analyzer);
        writer = new IndexWriter(dir, writerConfig);

        for (int i = 0; i < 100_000; i++) {
            final Document doc = new Document();
            doc.add(new BinaryDocValuesField(FIELD_NAME, generateBytesRef(true)));
            writer.addDocument(doc);
        }
        {
            floats = generateArray();
            final Document doc = new Document();
            doc.add(new BinaryDocValuesField(FIELD_NAME, toBytesRef(floats, true)));
            writer.addDocument(doc);
        }

        writer.commit();
    }

    @AfterClass
    public void tearDown() throws IOException {
        writer.close();
        dir.close();
    }

    @Test
    public void searchSimpleCache() throws IOException {
        final IndexReader reader = DirectoryReader.open(writer);
        final IndexSearcher searcher = new IndexSearcher(reader);
        for (float[] query_value : Arrays.asList(generateArray(), floats)) {
            final TopDocs d = searcher.search(new VQuery(FIELD_NAME, query_value, VScorerDefault::new), 100);
            assertNotNull(d.totalHits);
            final HashMap<Integer, float[]> cache = new HashMap<>();
            final TopDocs s = searcher.search(new VQuery(FIELD_NAME, query_value, (w, dv, dbase) -> new VScorerSimpleCache(w, dv, dbase, cache)), 100);
            assertNotNull(s.totalHits);
            assertEquals(s.scoreDocs[0].doc, d.scoreDocs[0].doc);
        }
    }

    @Test
    public void searchNative() throws IOException {
        final IndexReader reader = DirectoryReader.open(writer);
        final IndexSearcher searcher = new IndexSearcher(reader);
        final VScoreNative.ScorerFactory scorerFactory = new VScoreNative.ScorerFactory();
        for (float[] query_value : Arrays.asList(generateArray(), floats)) {
            final VQuery query = new VQuery(FIELD_NAME, query_value, scorerFactory);
            final TopDocs n = searcher.search(query, 100);
            query.close();
            assertNotNull(n.totalHits);
            final TopDocs d = searcher.search(new VQuery(FIELD_NAME, query_value, VScorerDefault::new), 100);
            assertNotNull(d.totalHits);
            assertEquals(n.scoreDocs[0].doc, d.scoreDocs[0].doc);
        }
        scorerFactory.close();
    }

}