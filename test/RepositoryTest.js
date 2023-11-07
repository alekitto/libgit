import { InitOptions, Repository, RepositoryState } from '../index';
import { randomBytes } from 'node:crypto';
import { tmpdir } from 'node:os';

const Filesystem = Jymfony.Component.Filesystem.Filesystem;
const File = Jymfony.Component.Filesystem.File;
const TestCase = Jymfony.Component.Testing.Framework.TestCase;

const fs = new Filesystem();

export default class RepositoryTest extends TestCase {
    _tmpDirName;

    async beforeEach() {
        this._tmpDirName = tmpdir() + '/' + randomBytes(5).toString('base64').replace(/[^0-9a-z]/i, '-');
        await fs.mkdir(this._tmpDirName);
    }

    async afterEach() {
        await fs.remove(this._tmpDirName);
    }

    async testRepositoryInit() {
        const repo = await Repository.init(this._tmpDirName);
        __self.assertFalse(repo.isBare());
        __self.assertTrue(await repo.isEmpty());
    }

    async testRepositoryInitBare() {
        const opts = new InitOptions();
        opts.setBare(true);

        const repo = await Repository.init(this._tmpDirName, opts);
        __self.assertTrue(repo.isBare());
        __self.assertTrue(await repo.isEmpty());
        __self.assertNull(await repo.namespace());
    }

    async testRepositoryOpen() {
        await Repository.init(this._tmpDirName);

        const repo = await Repository.open(this._tmpDirName);
        __self.assertFalse(repo.isBare());
        __self.assertTrue(await repo.isEmpty());
        __self.assertEquals(RepositoryState.Clean, await repo.state());
    }

    async testRepositoryOpenBare() {
        const opts = new InitOptions();
        opts.setBare(true);

        await Repository.init(this._tmpDirName, opts);

        const repo = await Repository.open(this._tmpDirName);
        __self.assertTrue(repo.isBare());
        __self.assertTrue(await repo.isEmpty());
        __self.assertEquals(RepositoryState.Clean, await repo.state());
    }

    async testRepositoryClone() {
        const opts = new InitOptions();
        opts.setInitialHead('master');

        const repo = await Repository.init(this._tmpDirName, opts);
        const config = await repo.config();
        config.setStr('user.name', 'test');
        config.setStr('user.email', 'test@example.com');

        const sig = await repo.signature();

        const file = new File(this._tmpDirName + '/README.md');
        const openFile = await file.openFile('w+');
        await openFile.fwrite(Buffer.from('Example\n'));
        await openFile.close();

        const index = await repo.index();
        await index.addPath('README.md');
        const tree_id = await index.writeTree();
        const tree = await repo.findTree(tree_id);

        const commit_id = await repo.createCommit(
            'refs/heads/master',
            sig,
            sig,
            "first commit",
            tree,
            [],
        );

        __self.assertNotNull(commit_id);

        const tmpDirName = tmpdir() + '/' + randomBytes(5).toString('base64').replace(/[^0-9a-z]/i, '-');
        try {
            await Repository.clone('file://' + this._tmpDirName, tmpDirName);
            const r2 = await Repository.open(tmpDirName);
            const commit = await r2.getBranchCommit('refs/heads/master');

            __self.assertEquals('first commit', commit.messageRaw());
        } finally {
            await fs.remove(tmpDirName);
        }
    }
}
