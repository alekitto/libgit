import { InitOptions, Repository, RepositoryState } from '../index';
import { randomBytes } from 'node:crypto';
import { sep } from 'node:path';
import { tmpdir } from 'node:os';

const Filesystem = Jymfony.Component.Filesystem.Filesystem;
const File = Jymfony.Component.Filesystem.File;
const TestCase = Jymfony.Component.Testing.Framework.TestCase;

const fs = new Filesystem();

export default class RepositoryTest extends TestCase {
    _tmpDirName;

    async beforeEach() {
        this._tmpDirName = tmpdir() + sep + randomBytes(5).toString('base64').replace(/[^0-9a-z]/i, '-');
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

        const file = new File(this._tmpDirName + sep + 'README.md');
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

        const tmpDirName = tmpdir() + sep + randomBytes(5).toString('base64').replace(/[^0-9a-z]/i, '-');
        try {
            await Repository.clone('file://' + this._tmpDirName, tmpDirName);
            const r2 = await Repository.open(tmpDirName);
            const commit = await r2.getBranchCommit('refs/heads/master');

            __self.assertEquals('first commit', commit.messageRaw());
        } finally {
            await fs.remove(tmpDirName);
        }
    }

    async testRepositoryFastRebase() {
        const opts = new InitOptions();
        opts.setInitialHead('master');

        const repo = await Repository.init(this._tmpDirName, opts);
        const config = await repo.config();
        config.setStr('user.name', 'test');
        config.setStr('user.email', 'test@example.com');

        const sig = await repo.signature();

        let file = new File(this._tmpDirName + sep + 'README.md');
        let openFile = await file.openFile('w+');
        await openFile.fwrite(Buffer.from('Example\n'));
        await openFile.close();

        let index = await repo.index();
        await index.addPath('README.md');
        let tree_id = await index.writeTree();
        let tree = await repo.findTree(tree_id);

        const commit_id = await repo.createCommit(
            'HEAD',
            sig,
            sig,
            "first commit",
            tree,
            [],
        );

        await repo.createBranch('branch_1', commit_id, false);
        await repo.createBranch('branch_2', commit_id, false);

        await repo.checkout('refs/heads/branch_1');

        file = new File(this._tmpDirName + sep + 'EXAMPLE.md');
        openFile = await file.openFile('w+');
        await openFile.fwrite(Buffer.from('Example\n'));
        await openFile.close();

        index = await repo.index();
        await index.addPath('EXAMPLE.md');
        tree_id = await index.writeTree();
        tree = await repo.findTree(tree_id);

        await repo.createCommit(
            'HEAD',
            sig,
            sig,
            "second commit",
            tree,
            [await repo.findCommit(commit_id)],
        );

        await repo.checkout('refs/heads/branch_2');

        file = new File(this._tmpDirName + sep + 'README.md');
        openFile = await file.openFile('w+');
        await openFile.fwrite(Buffer.from('Example\n\nFrom second branch'));
        await openFile.close();

        index = await repo.index();
        await index.addPath('README.md');
        tree_id = await index.writeTree();
        tree = await repo.findTree(tree_id);

        await repo.createCommit(
            'HEAD',
            sig,
            sig,
            "third commit",
            tree,
            [await repo.findCommit(commit_id)],
        );

        await repo.fastRebase('refs/heads/branch_1');
        const lastCommit = await repo.getBranchCommit('refs/heads/branch_2');
        __self.assertEquals('second commit', lastCommit.getParents()[0].messageRaw());

        // const tmpDirName = tmpdir() + sep + randomBytes(5).toString('base64').replace(/[^0-9a-z]/i, '-');
        // try {
        //     await Repository.clone('file://' + this._tmpDirName, tmpDirName);
        //     const r2 = await Repository.open(tmpDirName);
        //     const commit = await r2.getBranchCommit('refs/heads/master');
        //
        //     __self.assertEquals('first commit', commit.messageRaw());
        // } finally {
        //     await fs.remove(tmpDirName);
        // }
    }
}
