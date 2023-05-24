import { Repository, RepositoryState } from '../index';
import { mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';

const TestCase = Jymfony.Component.Testing.Framework.TestCase;

export default class RepositoryTest extends TestCase {
    testRepositoryInit() {
        const repo = Repository.init(mkdtempSync(tmpdir() + '/libgit-'));
        __self.assertFalse(repo.isBare());
        __self.assertTrue(repo.isEmpty());
    }

    testRepositoryInitBare() {
        const repo = Repository.init(mkdtempSync(tmpdir() + '/libgit-'), true);
        __self.assertTrue(repo.isBare());
        __self.assertTrue(repo.isEmpty());
        __self.assertNull(repo.namespace());
    }

    testRepositoryOpen() {
        const path = mkdtempSync(tmpdir() + '/libgit-');
        Repository.init(path);

        const repo = Repository.open(path);
        __self.assertFalse(repo.isBare());
        __self.assertTrue(repo.isEmpty());
        __self.assertEquals(RepositoryState.Clean, repo.state());
    }

    testRepositoryOpenBare() {
        const path = mkdtempSync(tmpdir() + '/libgit-');
        Repository.init(path, true);

        const repo = Repository.open(path);
        __self.assertTrue(repo.isBare());
        __self.assertTrue(repo.isEmpty());
        __self.assertEquals(RepositoryState.Clean, repo.state());
    }
}
