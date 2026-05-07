package lang.ktav

import com.intellij.lang.ASTNode
import com.intellij.lang.ParserDefinition
import com.intellij.lang.PsiParser
import com.intellij.lexer.Lexer
import com.intellij.openapi.project.Project
import com.intellij.psi.FileViewProvider
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiFile
import com.intellij.psi.TokenType
import com.intellij.psi.tree.IFileElementType
import com.intellij.psi.tree.TokenSet
import com.intellij.extapi.psi.ASTWrapperPsiElement
import com.intellij.extapi.psi.PsiFileBase
import lang.ktav.highlighting.KtavLexer
import lang.ktav.highlighting.KtavTokenTypes

/**
 * Минимальный ParserDefinition для Ktav.
 *
 * Цель: дать IntelliJ возможность создать PsiFile, чтобы заработали:
 *   - ExternalAnnotator (tooltip при наведении на ошибку)
 *   - Problems Tool Window
 *   - Find Usages базово
 *
 * Парсер строит плоское дерево: один root-node со всеми токенами как leaves.
 * Этого достаточно для tooltip — нам не нужна реальная иерархия (диагностики
 * приходят через LSP, а не через Inspection/Annotator на PSI).
 */
class KtavParserDefinition : ParserDefinition {

    override fun createLexer(project: Project?): Lexer = KtavLexer()

    override fun createParser(project: Project?): PsiParser = KtavParser()

    override fun getFileNodeType(): IFileElementType = FILE

    override fun getCommentTokens(): TokenSet = COMMENTS

    override fun getStringLiteralElements(): TokenSet = STRINGS

    override fun getWhitespaceTokens(): TokenSet = TokenSet.create(
        TokenType.WHITE_SPACE,
        KtavTokenTypes.WHITESPACE
    )

    override fun createElement(node: ASTNode): PsiElement = ASTWrapperPsiElement(node)

    override fun createFile(viewProvider: FileViewProvider): PsiFile = KtavPsiFile(viewProvider)

    companion object {
        val FILE = IFileElementType(KtavLanguage)
        val COMMENTS = TokenSet.create(KtavTokenTypes.COMMENT)
        val STRINGS = TokenSet.create(KtavTokenTypes.STRING)
    }
}

/**
 * Простейший parser: root-node, в который складывает все токены подряд.
 * Не строит иерархию — её для tooltip не требуется.
 */
class KtavParser : PsiParser {
    override fun parse(root: com.intellij.psi.tree.IElementType, builder: com.intellij.lang.PsiBuilder): ASTNode {
        val rootMarker = builder.mark()
        while (!builder.eof()) {
            builder.advanceLexer()
        }
        rootMarker.done(root)
        return builder.treeBuilt
    }
}

/**
 * PsiFile реализация для .ktav.
 */
class KtavPsiFile(viewProvider: FileViewProvider) :
    PsiFileBase(viewProvider, KtavLanguage) {
    override fun getFileType() = KtavFileType
    override fun toString(): String = "Ktav File"
}
